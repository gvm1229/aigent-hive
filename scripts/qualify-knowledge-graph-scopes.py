#!/usr/bin/env python3
"""Run unchanged gold questions across source, project and user-root knowledge."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from tests.conformance.support.harness import write_operational_user_setup

spec = importlib.util.spec_from_file_location("source_gold", ROOT / "scripts/qualify-source-graph.py")
gold = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gold)


def digest_tree(root: Path) -> str:
    values = [(p.name, hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted(root.glob("*.md"))]
    return hashlib.sha256(json.dumps(values).encode()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hive", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    work = args.work.resolve()
    owned = (ROOT / "tests/work").resolve()
    if work == owned or not work.is_relative_to(owned) or work.exists():
        raise SystemExit("use a fresh exact child of tests/work")
    work.mkdir(parents=True)
    binary = args.hive.resolve()
    source, project, user = (work / name for name in ("source", "project", "user"))
    source.mkdir()
    project.mkdir()
    gold.frozen_source(ROOT, source)
    write_operational_user_setup(user)
    gold.invoke(binary, ["setup", "--target", str(project), "--user-root", str(user),
        "--answers", str(ROOT / "tests/fixtures/setup/answers-no-role-no-hook.yml"),
        "--capabilities", str(ROOT / "tests/fixtures/setup/capabilities-codex-host-native.json"),
        "--apply"], work)
    # Only the schema changes for consumer copies; facts, links and expectations stay fixed.
    for target in (project, user):
        wiki = target / ".hive/knowledge/Wiki"
        wiki.mkdir(parents=True, exist_ok=True)
        for fact in sorted((source / "docs/facts/en").glob("*.md")):
            raw = fact.read_bytes()
            fingerprint = hashlib.sha256(raw).hexdigest()
            raw_path = f".hive/knowledge/Raw/frozen/{fingerprint}.md"
            destination = target / raw_path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(raw)
            _, header, body = raw.decode().split("---", 2)
            original = yaml.safe_load(header)
            metadata = {"schema_version": 1, "id": original["pair_id"], "kind": "concept",
                "summary": original["title"] + ". " + original["summary"],
                "tags": original["tags"], "aliases": original["aliases"],
                "sources": [f"raw:{raw_path}#sha256:{fingerprint}"], "links": original["links"],
                "contradictions": [], "status": "active", "created_at": "2026-10-05T00:00:00Z",
                "updated_at": "2026-10-05T00:00:00Z"}
            text = "---\n" + yaml.safe_dump(metadata, sort_keys=False, allow_unicode=True) + "---" + body
            (wiki / (metadata["id"] + ".md")).write_text(text, encoding="utf-8", newline="\n")
    gold.invoke(binary, ["index", "rebuild", "--user-root", str(user)], work)
    gold.invoke(binary, ["source-wiki", "index", "--target", str(source)], work)
    corpus_path = ROOT / "tests/fixtures/knowledge/vector-gold-120.json"
    corpus_bytes = corpus_path.read_bytes()
    queries = json.loads(corpus_bytes)["queries"]
    exact = [q for q in queries if q["kind"] == "exact"]
    relation = [q for q in queries if q["kind"] == "relation"]
    assert len(exact) == len(relation) == 30
    before = [gold.tree_digest(source), digest_tree(project / ".hive/knowledge/Wiki"),
              digest_tree(user / ".hive/knowledge/Wiki")]
    report = {"schema_version": 1, "engine": "native-markdown", "host": sys.platform,
        "facts_revision": gold.CORPUS_FACTS_REVISION,
        "question_corpus_sha256": hashlib.sha256(corpus_bytes).hexdigest(),
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "semantic_model_accuracy": "not measured by this explicit-link regression", "scopes": []}
    total_exact = total_relation = 0
    latencies = []
    for number, (label, target) in enumerate((("source", source), ("project", project), ("user-root", user))):
        row = {"scope": label, "exact_passed": 0, "relation_passed": 0, "failures": []}
        for kind, selected in (("exact", exact), ("relation", relation)):
            for item in selected[number*10:(number+1)*10]:
                prefix = ["source-wiki"] if label == "source" else ["knowledge"]
                command = prefix + (["query"] if kind == "exact" else ["graph", "query"])
                command += ["--target", str(target), "--text", item["query"]]
                if label != "source":
                    command += ["--user-root", str(user)]
                elif kind == "exact":
                    command += ["--language", "en"]
                if kind == "exact":
                    command += ["--limit", "10"]
                result, elapsed = gold.invoke(binary, command, work)
                latencies.append(elapsed)
                expected = set(item["expected"])
                passed = bool(expected & gold.hit_ids(result, graph=kind == "relation"))
                if kind == "relation":
                    endpoint = item["query"].rsplit(" relate to ", 1)[1].removesuffix("?")
                    passed = passed and any(e["evidence"] == "EXTRACTED" and
                        ((e["from"] in expected and e["to"] == endpoint) or
                         (e["to"] in expected and e["from"] == endpoint))
                        for e in result["data"]["matches"])
                row[kind + "_passed"] += int(passed)
                if not passed:
                    row["failures"].append(item["id"])
        total_exact += row["exact_passed"]
        total_relation += row["relation_passed"]
        report["scopes"].append(row)
    after = [gold.tree_digest(source), digest_tree(project / ".hive/knowledge/Wiki"),
             digest_tree(user / ".hive/knowledge/Wiki")]
    report.update(exact_passed=total_exact, relation_passed=total_relation,
        cold_cli_p95_ms=gold.percentile(latencies, .95), canonical_preserved=before == after,
        questions_preserved=corpus_path.read_bytes() == corpus_bytes)
    report["passed"] = (total_exact == 30 and total_relation >= 27
        and report["cold_cli_p95_ms"] <= 2000 and before == after and report["questions_preserved"])
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(report))
    return int(not report["passed"])


if __name__ == "__main__":
    raise SystemExit(main())
