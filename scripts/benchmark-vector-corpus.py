#!/usr/bin/env python3
"""Measure the deterministic FTS baseline for the vector feasibility corpus."""

from __future__ import annotations

import argparse
import json
import re
import sqlite3
import statistics
import time
from pathlib import Path


def percentile(values: list[float], percent: float) -> float:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(len(ordered) * percent))]


def match_expression(text: str) -> str:
    tokens = list(dict.fromkeys(re.findall(r"[^\W_]+", text.casefold(), flags=re.UNICODE)))
    return " OR ".join(f'"{token.replace(chr(34), chr(34) * 2)}"' for token in tokens[:64])


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repetitions", type=int, default=5)
    args = parser.parse_args()
    corpus = json.loads(args.corpus.read_text(encoding="utf-8"))
    connection = sqlite3.connect(":memory:")
    connection.execute("CREATE VIRTUAL TABLE docs USING fts5(id UNINDEXED, text, tokenize='unicode61')")
    connection.executemany("INSERT INTO docs(id,text) VALUES(?,?)", ((item["id"], item["text"]) for item in corpus["documents"]))
    connection.commit()
    latencies = []
    returned_bytes = []
    ranks = []
    by_kind: dict[str, list[int | None]] = {}
    for query in corpus["queries"]:
        expression = match_expression(query["query"])
        result = []
        for _ in range(args.repetitions):
            started = time.perf_counter_ns()
            result = connection.execute(
                "SELECT id FROM docs WHERE docs MATCH ? ORDER BY bm25(docs) LIMIT 10",
                (expression,),
            ).fetchall()
            latencies.append((time.perf_counter_ns() - started) / 1_000_000)
        ids = [row[0] for row in result]
        returned_bytes.append(len(json.dumps(ids, separators=(",", ":")).encode()))
        expected = set(query["expected"])
        rank = next((index for index, item in enumerate(ids, 1) if item in expected), None)
        ranks.append(rank)
        by_kind.setdefault(query["kind"], []).append(rank)
    metrics = {
        "schema_version": 1,
        "engine": "sqlite-fts5-unicode61-bm25",
        "document_count": len(corpus["documents"]),
        "query_count": len(corpus["queries"]),
        "recall_at_10": sum(rank is not None for rank in ranks) / len(ranks),
        "mrr": sum(0 if rank is None else 1 / rank for rank in ranks) / len(ranks),
        "warm_p95_ms": percentile(latencies, 0.95),
        "warm_median_ms": statistics.median(latencies),
        "returned_bytes_p95": percentile([float(value) for value in returned_bytes], 0.95),
        "by_kind": {
            kind: {
                "count": len(values),
                "recall_at_10": sum(value is not None for value in values) / len(values),
                "mrr": sum(0 if value is None else 1 / value for value in values) / len(values),
            }
            for kind, values in sorted(by_kind.items())
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(metrics, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(metrics, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
