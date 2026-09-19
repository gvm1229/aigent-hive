#!/usr/bin/env python3
"""Compare preserved CLI binaries with the existing, fixed user-root bilingual gold subset."""

import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
GOLD = ROOT / "crates/hive-wiki/tests/knowledge_retrieval_qualification.rs"


def digest(value):
    return "sha256:" + hashlib.sha256(value).hexdigest()


def snapshot(root):
    return {p.relative_to(root).as_posix(): digest(p.read_bytes())
            for p in sorted(root.rglob("*")) if p.is_file()}


def invoke(binary, arguments, expected=0):
    started = time.perf_counter()
    result = subprocess.run([str(binary), *map(str, arguments), "--output", "json"],
                            cwd=ROOT, capture_output=True, timeout=30)
    elapsed = (time.perf_counter() - started) * 1000
    payload = json.loads(result.stdout)
    if result.returncode != expected:
        raise ValueError(f"{arguments[0:2]}: {result.returncode}, {payload.get('code')}")
    return payload, elapsed


def cases():
    text = GOLD.read_text(encoding="utf-8").split("const BILINGUAL_GOLD:", 1)[1].split("\n];", 1)[0]
    result = []
    for block in re.findall(r"GoldCase \{(.*?)\n    \}", text, re.S):
        if "location: GoldLocation::UserRoot" not in block:
            continue
        row = {key: json.loads(value) for key, value in
               re.findall(r'(key|english_query|korean_query|fact): ("(?:[^"\\]|\\.)*")', block)}
        row["kind"] = re.search(r"kind: ClaimKind::(\w+)", block)[1].lower()
        result.append(row)
    if len(result) != 11 or any(len(row) != 5 for row in result):
        raise ValueError("fixed gold subset changed; review the comparator")
    return result


def run(binary, work, gold):
    user = work / "user"
    target = work / "unregistered-project"
    target.mkdir(parents=True)
    config = user / ".hive/config/user-setup.yml"
    config.parent.mkdir(parents=True)
    config.write_text(json.dumps({
        "schema_version": 1, "interface_language": "en",
        "wiki": {"enabled": True, "language": "both"},
        "profile": {"contexts": ["web-developer"]}, "persona": {"id": "balanced"},
        "selected_hosts": ["codex"], "skills": {"mode": "all"},
        "usage_guard": {"enabled": True, "stop_remaining_percent": 20,
                        "codexbar_fallback_enabled": False, "discord": {"enabled": False}},
    }), encoding="utf-8")
    ids = {}
    remembered = []
    for row in gold:
        key = row["key"]
        request = {
            "collection_id": "user-root", "claim_key": key, "claim_id": None,
            "locator": f"pending/{key}.md", "kind": row["kind"], "status": "user-stated",
            "visibility": "shared", "normalized_fact": row["fact"],
            "provenance": {"source_kind": "user-statement", "summary": f"Reviewed durable statement for {key}",
                           "locator": f"request:{key}", "digest": digest(row["fact"].encode())},
            "sources": [f"request:{key}"], "supersedes": [], "expected_active_digest": None,
            "observed_at": None, "verified_at": None,
        }
        path = work / f"{key}.json"
        path.write_text(json.dumps(request, ensure_ascii=False), encoding="utf-8")
        args = ["knowledge", "remember", "--user-root", user, "--request", path]
        inserted, _ = invoke(binary, args)
        ids[key] = inserted["data"]["plan"]["new_claim"]["claim_id"]
        before = snapshot(user)
        repeated, _ = invoke(binary, args)
        if snapshot(user) != before or repeated["changed_paths"]:
            raise ValueError("repeated memory changed canonical or derived bytes")
        remembered.append({"key": key, "action": inserted["action"], "code": inserted["code"],
                           "repeat_code": repeated["code"], "repeat_unchanged": True})
    before_queries = snapshot(user)
    rows = []
    for row in gold:
        for language in ("english", "korean"):
            samples, results = [], []
            for iteration in range(35):
                result, elapsed = invoke(binary, ["knowledge", "retrieve", "--user-root", user,
                    "--target", target, "--scope", "auto", "--query", row[f"{language}_query"],
                    "--top-k", "5", "--byte-budget", "16384"])
                hit_ids = [hit["item_id"] for hit in result["data"]["hits"]]
                results.append(hit_ids)
                if iteration >= 5:
                    samples.append(elapsed)
            if any(ids[row["key"]] not in hits for hits in results):
                raise ValueError("expected gold claim missing from top five")
            if any(hits != results[0] for hits in results):
                raise ValueError("retrieval order changed across fresh CLI processes")
            rows.append({"key": row["key"], "language": language, "hit_ids": results[0],
                         "median_ms": statistics.median(samples), "p95_ms": sorted(samples)[28],
                         "variance_ms2": statistics.pvariance(samples), "samples_ms": samples})
    if snapshot(user) != before_queries or list(target.iterdir()):
        raise ValueError("read-only retrieval changed the user or unregistered target")
    denied, _ = invoke(binary, ["knowledge", "remember", "--user-root", user,
        "--user-statement", "token sk-abcdefghijklmnopqrstuvwxyz0123456789",
        "--claim-key", "fixture.secret", "--kind", "preference"], expected=2)
    if snapshot(user) != before_queries:
        raise ValueError("rejected capture changed stored bytes")
    print(f"{work.name}: 22/22 queries, duplicate and secret rejection unchanged", flush=True)
    return {"binary_digest": digest(binary.read_bytes()), "remembered": remembered,
            "queries": rows, "secret_rejection": denied["code"], "files_after": before_queries,
            "queries_write_zero": True, "caller_questions": 0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before", type=Path, required=True)
    parser.add_argument("--after", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    work = args.work.resolve()
    if not work.is_relative_to((ROOT / "tests/work").resolve()) or work.exists():
        raise ValueError("use a new exact directory under tests/work; never overwrite comparison evidence")
    work.mkdir(parents=True)
    gold = cases()
    report = {"schema_version": 1, "platform": platform.platform(), "gold_digest": digest(GOLD.read_bytes()),
        "gold_subset": "11 existing user-root facts, 22 bilingual queries; project cases remain in Rust qualification",
        "warmups": 5, "samples": 30, "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "limit": "synthetic CLI processes only, not live host conversations, project-private scope, or large-corpus performance"}
    for label, binary in (("before", args.before), ("after", args.after)):
        report[label] = run(binary.resolve(), work / label, gold)
        (work / "comparison.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    for before, after in zip(report["before"]["queries"], report["after"]["queries"], strict=True):
        if before["hit_ids"] != after["hit_ids"]:
            raise ValueError("gold results differ between binaries")
    report["same_retrieval_results"] = True
    (work / "comparison.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("Before/after fixed-query results are identical; timings retained without changing thresholds.")


if __name__ == "__main__":
    main()
