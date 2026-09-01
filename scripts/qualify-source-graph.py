#!/usr/bin/env python3
"""Qualify the shipped source graph CLI with the fixed 30+30 gold questions."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import tempfile
import time
import zipfile


CORPUS_FACTS_REVISION = "622f2b7b5411d054abd94c5443ce2b620231b240"


def frozen_source(repository: Path, target: Path) -> None:
    """Copy fixed facts and their cited bytes, never execute historical code."""
    payload = subprocess.check_output(
        ["git", "archive", "--format=zip", CORPUS_FACTS_REVISION],
        cwd=repository, timeout=30,
    )
    with zipfile.ZipFile(io.BytesIO(payload)) as archive:
        facts = [name for name in archive.namelist()
                 if re.fullmatch(r"docs/facts/(en|ko)/[^/]+\.md", name)]
        if not facts:
            raise ValueError("frozen source facts are missing")
        selected = {"hive-source.json", *facts}
        for name in facts:
            selected.update(match.decode("utf-8") for match in re.findall(
                rb'^\s+- "repo:([^"#]+)#sha256:[0-9a-f]{64}"\s*$',
                archive.read(name), re.MULTILINE,
            ))
        for name in sorted(selected):
            path = PurePosixPath(name)
            info = archive.getinfo(name)
            if (path.is_absolute() or any(part in (".", "..") for part in path.parts)
                    or "\\" in name or ":" in name or info.is_dir()
                    or (info.external_attr >> 16) & 0o170000 == 0o120000):
                raise ValueError("frozen source contains an unsafe file")
            destination = target.joinpath(*path.parts)
            destination.parent.mkdir(parents=True, exist_ok=True)
            with destination.open("xb") as output:
                output.write(archive.read(name))


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
    repository = args.target.resolve()
    binary = args.hive.resolve()
    corpus_path = (repository / args.corpus).resolve() if not args.corpus.is_absolute() else args.corpus
    corpus_bytes = corpus_path.read_bytes()
    corpus_digest = "sha256:" + hashlib.sha256(corpus_bytes).hexdigest()
    corpus = json.loads(corpus_bytes)
    exact = [item for item in corpus["queries"] if item["kind"] == "exact"]
    relation = [item for item in corpus["queries"] if item["kind"] == "relation"]
    if len(exact) != 30 or len(relation) != 30:
        raise SystemExit("source graph qualification requires exactly 30 exact and 30 relation questions")

    # Current truth is still validated. Historical questions are not silently
    # rewritten to fit changing facts, nor are facts reverted to pass a benchmark.
    current_before = tree_digest(repository)
    invoke(binary, ["source-wiki", "index", "--target", str(repository)], repository)
    current_lint, _ = invoke(binary, ["source-wiki", "lint", "--target", str(repository)], repository)
    invoke(binary, ["source-wiki", "graph", "rebuild", "--target", str(repository)], repository)
    work = repository / "tests/work"
    work.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="source-graph-gold-", dir=work) as directory:
        target = Path(directory).resolve()
        frozen_source(repository, target)
        return qualify(args, binary, target, repository, corpus_path, corpus_digest, exact, relation,
                       current_before, current_lint)


def qualify(args, binary, target, repository, corpus_path, corpus_digest, exact, relation,
            current_before, current_lint) -> int:

    before = tree_digest(target)
    invoke(binary, ["source-wiki", "index", "--target", str(target)], target)
    frozen_lint, _ = invoke(binary, ["source-wiki", "lint", "--target", str(target)], target)
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
                str(repository),
                "--engine",
                "graphify-code",
            ],
            repository,
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
        "facts_revision": CORPUS_FACTS_REVISION,
        "question_corpus_digest": corpus_digest,
        "question_corpus_changed": "sha256:" + hashlib.sha256(corpus_path.read_bytes()).hexdigest() != corpus_digest,
        "current_source_lint": current_lint,
        "frozen_source_lint": frozen_lint,
        "current_canonical_changed": tree_digest(repository) != current_before,
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
        or report["current_canonical_changed"]
        or report["question_corpus_changed"]
        or current_lint["data"]["error_count"] != 0
        or current_lint["data"]["warning_count"] != 0
        or frozen_lint["data"]["error_count"] != 0
        or frozen_lint["data"]["warning_count"] != 0
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
