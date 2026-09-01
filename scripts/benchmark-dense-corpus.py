#!/usr/bin/env python3
"""Measure local multilingual dense retrieval quality without a vector database."""

from __future__ import annotations

import argparse
import hashlib
import json
import statistics
import time
from pathlib import Path

import numpy as np
from fastembed import TextEmbedding


def tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        digest.update(path.relative_to(root).as_posix().encode())
        digest.update(b"\0")
        digest.update(hashlib.sha256(path.read_bytes()).digest())
    return "sha256:" + digest.hexdigest()


def normalized(values: list[np.ndarray]) -> np.ndarray:
    matrix = np.asarray(values, dtype=np.float32)
    norms = np.linalg.norm(matrix, axis=1, keepdims=True)
    return matrix / np.maximum(norms, 1e-12)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--cache", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--model",
        default="sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
    )
    parser.add_argument("--scale-count", type=int, default=0)
    args = parser.parse_args()
    corpus = json.loads(args.corpus.read_text(encoding="utf-8"))
    args.cache.mkdir(parents=True, exist_ok=True)
    started = time.perf_counter()
    model = TextEmbedding(model_name=args.model, cache_dir=str(args.cache))
    documents = normalized(list(model.embed([item["text"] for item in corpus["documents"]])))
    queries = normalized(list(model.query_embed([item["query"] for item in corpus["queries"]])))
    build_seconds = time.perf_counter() - started
    latencies = []
    ranks = []
    by_kind: dict[str, list[int | None]] = {}
    ids = [item["id"] for item in corpus["documents"]]
    for query, vector in zip(corpus["queries"], queries, strict=True):
        started = time.perf_counter_ns()
        order = np.argsort(-(documents @ vector))[:10]
        latencies.append((time.perf_counter_ns() - started) / 1_000_000)
        result = [ids[index] for index in order]
        expected = set(query["expected"])
        rank = next((index for index, item in enumerate(result, 1) if item in expected), None)
        ranks.append(rank)
        by_kind.setdefault(query["kind"], []).append(rank)
    result = {
        "schema_version": 1,
        "engine": "numpy-exact-cosine",
        "model": args.model,
        "model_tree_digest": tree_digest(args.cache),
        "dimension": int(documents.shape[1]),
        "document_count": len(ids),
        "query_count": len(ranks),
        "build_seconds": build_seconds,
        "recall_at_10": sum(rank is not None for rank in ranks) / len(ranks),
        "mrr": sum(0 if rank is None else 1 / rank for rank in ranks) / len(ranks),
        "lookup_median_ms": statistics.median(latencies),
        "lookup_p95_ms": sorted(latencies)[min(len(latencies) - 1, int(len(latencies) * 0.95))],
        "by_kind": {
            kind: {
                "count": len(values),
                "recall_at_10": sum(value is not None for value in values) / len(values),
                "mrr": sum(0 if value is None else 1 / value for value in values) / len(values),
            }
            for kind, values in sorted(by_kind.items())
        },
    }
    if args.scale_count:
        source_documents = [item["text"] for item in corpus["documents"]]
        scaled = [source_documents[index % len(source_documents)] for index in range(args.scale_count)]
        started = time.perf_counter()
        scaled_count = sum(1 for _ in model.embed(scaled, batch_size=256))
        result["scale_build"] = {
            "count": scaled_count,
            "seconds": time.perf_counter() - started,
        }
        started = time.perf_counter()
        incremental_count = sum(1 for _ in model.embed(scaled[:100], batch_size=100))
        result["incremental_build"] = {
            "count": incremental_count,
            "seconds": time.perf_counter() - started,
        }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
