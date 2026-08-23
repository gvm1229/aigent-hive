#!/usr/bin/env python3
"""Qualify the shipped source graph CLI with the fixed 30+30 gold questions."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time


def tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted((root / "docs" / "facts").rglob("*.md")):
        relative = path.relative_to(root).as_posix().encode()
        payload = path.read_bytes()
        digest.update(len(relative).to_bytes(8, "big"))
        digest.update(relative)
        digest.update(len(payload).to_bytes(8, "big"))
        digest.update(payload)
    return "sha256:" + digest.hexdigest()


def invoke(binary: Path, arguments: list[str], cwd: Path) -> tuple[dict[str, object], float]:
    started = time.perf_counter()
    completed = subprocess.run(
        [str(binary), *arguments, "--output", "json"],
        cwd=cwd,
        check=True,
        capture_output=True,
        text=True,
        timeout=10,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000
    return json.loads(completed.stdout), elapsed_ms


def percentile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(len(ordered) * fraction))]


def hit_ids(result: dict[str, object], *, graph: bool) -> set[str]:
    data = result["data"]
    assert isinstance(data, dict)
    if graph:
        data = data["fts"]
        assert isinstance(data, dict)
    hits = data["hits"]
    assert isinstance(hits, list)
    return {
        str(hit.get("pair_id", hit.get("id")))
        for hit in hits
        if isinstance(hit, dict)
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hive", type=Path, required=True)
    parser.add_argument("--target", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, default=Path("tests/fixtures/knowledge/vector-gold-120.json"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--require-graphify-preview", action="store_true")
    args = parser.parse_args()
    target = args.target.resolve()
    binary = args.hive.resolve()
    corpus_path = (target / args.corpus).resolve() if not args.corpus.is_absolute() else args.corpus
    corpus = json.loads(corpus_path.read_text(encoding="utf-8"))
    exact = [item for item in corpus["queries"] if item["kind"] == "exact"]
    relation = [item for item in corpus["queries"] if item["kind"] == "relation"]
    if len(exact) != 30 or len(relation) != 30:
        raise SystemExit("source graph qualification requires exactly 30 exact and 30 relation questions")

    before = tree_digest(target)
    invoke(binary, ["source-wiki", "index", "--target", str(target)], target)
    rebuild, _ = invoke(
        binary,
        ["source-wiki", "graph", "rebuild", "--target", str(target)],
        target,
    )
    graphify_preview = False
    if args.require_graphify_preview:
        preview, _ = invoke(
            binary,
            [
                "source-wiki",
                "graph",
                "preview",
                "--target",
                str(target),
                "--engine",
                "graphify-code",
            ],
            target,
        )
        preview_data = preview["data"]
        assert isinstance(preview_data, dict)
        graphify_preview = (
            preview.get("status") == "success"
            and preview_data.get("package_version") == "0.9.47"
            and preview_data.get("provider_api_calls") == 0
            and preview_data.get("api_keys_read") == 0
            and preview_data.get("query_logs") == 0
            and preview_data.get("network_requests") == 0
            and str(preview_data.get("dependency_lock_digest", "")).startswith("sha256:")
        )
    exact_passes = 0
    relation_passes = 0
    failed_exact = []
    failed_relation = []
    latencies = []
    for item in exact:
        result, elapsed = invoke(
            binary,
            [
                "source-wiki",
                "query",
                "--target",
                str(target),
                "--language",
                "en",
                "--text",
                item["query"],
                "--limit",
                "10",
            ],
            target,
        )
        latencies.append(elapsed)
        passed = bool(set(item["expected"]) & hit_ids(result, graph=False))
        exact_passes += passed
        if not passed:
            failed_exact.append(item["id"])
    for item in relation:
        result, elapsed = invoke(
            binary,
            [
                "source-wiki",
                "graph",
                "query",
                "--target",
                str(target),
                "--text",
                item["query"],
            ],
            target,
        )
        latencies.append(elapsed)
        expected = set(item["expected"])
        data = result["data"]
        assert isinstance(data, dict)
        edges = data["matches"]
        assert isinstance(edges, list)
        grounded = any(
            isinstance(edge, dict)
            and (edge.get("from") in expected or edge.get("to") in expected)
            and edge.get("evidence") == "EXTRACTED"
            for edge in edges
        )
        passed = bool(expected & hit_ids(result, graph=True)) and grounded
        relation_passes += passed
        if not passed:
            failed_relation.append(item["id"])
    after = tree_digest(target)
    data = rebuild["data"]
    assert isinstance(data, dict)
    report = {
        "schema_version": 1,
        "questions": {"exact": 30, "relation": 30},
        "exact_recall_at_10": exact_passes / 30,
        "relation_grounded_recall_at_10": relation_passes / 30,
        "failed_exact": failed_exact,
        "failed_relation": failed_relation,
        "cold_cli_p95_ms": round(percentile(latencies, 0.95), 4),
        "canonical_tree_digest_before": before,
        "canonical_tree_digest_after": after,
        "canonical_changed": before != after,
        "scope": data.get("scope"),
        "engine": data.get("engine"),
        "node_count": data.get("node_count"),
        "edge_count": data.get("edge_count"),
        "provider_api_calls": 0,
        "api_keys_read": 0,
        "query_logs": 0,
        "graphify_preview": graphify_preview,
    }
    if (
        report["exact_recall_at_10"] < 1.0
        or report["relation_grounded_recall_at_10"] < 0.9
        or report["cold_cli_p95_ms"] > 2000
        or report["canonical_changed"]
        or report["scope"] != "source"
        or report["engine"] != "native-markdown"
        or (args.require_graphify_preview and not report["graphify_preview"])
    ):
        raise SystemExit("source graph qualification gate failed: " + json.dumps(report, sort_keys=True))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(report, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
