#!/usr/bin/env python3
"""Benchmark isolated SQLite vector engines with one deterministic 50k corpus."""

from __future__ import annotations

import argparse
import json
import pathlib
import sqlite3
import statistics
import time

import numpy as np
import psutil
import sqlite_vec
import sqlite_vector


def percentile(values: list[float], percent: float) -> float:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(len(ordered) * percent))]


def vectors(count: int, dimension: int) -> np.ndarray:
    values = np.random.default_rng(20260823).normal(size=(count, dimension)).astype(np.float32)
    values /= np.maximum(np.linalg.norm(values, axis=1, keepdims=True), 1e-12)
    return values


def sqlite_vec_benchmark(path: pathlib.Path, values: np.ndarray, query_ids: list[int]) -> dict[str, object]:
    connection = sqlite3.connect(path)
    connection.enable_load_extension(True)
    sqlite_vec.load(connection)
    dimension = values.shape[1]
    started = time.perf_counter()
    connection.execute(f"CREATE VIRTUAL TABLE vectors USING vec0(embedding float[{dimension}])")
    connection.executemany(
        "INSERT INTO vectors(rowid,embedding) VALUES(?,?)",
        ((index + 1, sqlite_vec.serialize_float32(value)) for index, value in enumerate(values)),
    )
    connection.commit()
    build_seconds = time.perf_counter() - started
    latencies = []
    hits = 0
    for index in query_ids:
        query = sqlite_vec.serialize_float32(values[index])
        started = time.perf_counter_ns()
        result = connection.execute(
            "SELECT rowid FROM vectors WHERE embedding MATCH ? AND k=10",
            (query,),
        ).fetchall()
        latencies.append((time.perf_counter_ns() - started) / 1_000_000)
        hits += index + 1 in {row[0] for row in result}
    connection.execute("DELETE FROM vectors WHERE rowid BETWEEN 1 AND 100")
    connection.executemany(
        "INSERT INTO vectors(rowid,embedding) VALUES(?,?)",
        ((index + 1, sqlite_vec.serialize_float32(values[index])) for index in range(100)),
    )
    connection.commit()
    connection.close()
    return {
        "engine": "sqlite-vec==0.1.9",
        "build_seconds": build_seconds,
        "lookup_median_ms": statistics.median(latencies),
        "lookup_p95_ms": percentile(latencies, 0.95),
        "self_recall_at_10": hits / len(query_ids),
        "disk_bytes": path.stat().st_size,
    }


def sqlite_vector_benchmark(path: pathlib.Path, values: np.ndarray, query_ids: list[int]) -> dict[str, object]:
    connection = sqlite3.connect(path)
    connection.enable_load_extension(True)
    extension = pathlib.Path(sqlite_vector.__path__[0]) / "binaries" / "vector.dll"
    connection.load_extension(str(extension))
    dimension = values.shape[1]
    started = time.perf_counter()
    connection.execute("CREATE TABLE vectors(id INTEGER PRIMARY KEY, embedding BLOB NOT NULL)")
    connection.execute(
        f"SELECT vector_init('vectors','embedding','type=FLOAT32,dimension={dimension},distance=COSINE')"
    )
    connection.executemany(
        "INSERT INTO vectors(id,embedding) VALUES(?,?)",
        ((index + 1, value.tobytes()) for index, value in enumerate(values)),
    )
    connection.commit()
    connection.execute("SELECT vector_quantize('vectors','embedding')").fetchone()
    connection.commit()
    build_seconds = time.perf_counter() - started
    latencies = []
    hits = 0
    sql = "SELECT vectors.id FROM vectors JOIN vector_quantize_scan('vectors','embedding',?,10) AS q ON vectors.rowid=q.rowid"
    for index in query_ids:
        started = time.perf_counter_ns()
        result = connection.execute(sql, (values[index].tobytes(),)).fetchall()
        latencies.append((time.perf_counter_ns() - started) / 1_000_000)
        hits += index + 1 in {row[0] for row in result}
    connection.execute("DELETE FROM vectors WHERE id BETWEEN 1 AND 100")
    connection.executemany(
        "INSERT INTO vectors(id,embedding) VALUES(?,?)",
        ((index + 1, values[index].tobytes()) for index in range(100)),
    )
    connection.commit()
    connection.close()
    return {
        "engine": "sqliteai-vector==1.0.0",
        "build_seconds": build_seconds,
        "lookup_median_ms": statistics.median(latencies),
        "lookup_p95_ms": percentile(latencies, 0.95),
        "self_recall_at_10": hits / len(query_ids),
        "disk_bytes": path.stat().st_size,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    parser.add_argument("--count", type=int, default=50_000)
    parser.add_argument("--dimension", type=int, default=384)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    values = vectors(args.count, args.dimension)
    query_ids = np.random.default_rng(17).choice(args.count, size=100, replace=False).tolist()
    rss_before = psutil.Process().memory_info().rss
    results = [
        sqlite_vec_benchmark(args.output_dir / "sqlite-vec.sqlite3", values, query_ids),
        sqlite_vector_benchmark(args.output_dir / "sqlite-vector.sqlite3", values, query_ids),
    ]
    payload = {
        "schema_version": 1,
        "count": args.count,
        "dimension": args.dimension,
        "query_count": len(query_ids),
        "rss_delta_bytes": max(0, psutil.Process().memory_info().rss - rss_before),
        "results": results,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(payload, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
