#!/usr/bin/env python3
"""Measure frozen project labels once; never adjust the detector using these results."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
LABELS = ROOT / "tests/fixtures/project-detection/holdout/labels.json"
FROZEN_COMMIT = "0ee70b1d"


def score_facts(profile: dict, allowed: list[str], target: Path) -> tuple[int, list[dict]]:
    facts = [fact for group in ("languages", "frameworks", "package_managers", "test_tools", "structure") for fact in profile[group]]
    facts += list(profile["commands"].values())
    wrong = [fact for fact in facts if fact["name"] not in allowed or not fact["evidence"]
             or any(not (target / evidence).is_file() for evidence in fact["evidence"])]
    return len(facts), wrong


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hive", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    binary = args.hive.resolve(strict=True)
    work = args.work.resolve()
    if not work.is_relative_to(ROOT / "tests/work") or work.exists():
        parser.error("--work must be a new directory below tests/work")
    frozen = subprocess.check_output(["git", "show", f"{FROZEN_COMMIT}:tests/fixtures/project-detection/holdout/labels.json"], cwd=ROOT)
    assert frozen == LABELS.read_bytes(), "frozen labels changed"
    cases = json.loads(frozen)["cases"]
    work.mkdir(parents=True)
    user = work / "empty-user"
    user.mkdir()
    # The metric counts a named but unsupported fact as wrong, even with a real file.
    empty = {key: [] for key in ("languages", "frameworks", "package_managers", "test_tools", "structure")}
    empty["commands"] = {}
    assert score_facts(empty, [], user) == (0, [])
    total = wrong_count = correct = 0
    rows = []
    for case in cases:
        target = work / case["id"]
        target.mkdir()
        for name, text in case["files"].items():
            path = target / name
            assert path.resolve().is_relative_to(target.resolve())
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(text.encode())
        process = subprocess.run([str(binary), "setup", "--target", str(target), "--user-root", str(user), "--describe", "--question", "project-kind", "--output", "json"], capture_output=True, text=True, encoding="utf-8", check=True)
        value = json.loads(process.stdout)["data"]
        profile = value["project_profile"]
        kind = value["questions"][0]["question"]["default"]
        count, wrong = score_facts(profile, case["allowed_facts"], target)
        total += count
        wrong_count += len(wrong)
        correct += kind == case["expected_kind"]
        assert all((target / name).read_bytes() == text.encode() for name, text in case["files"].items())
        rows.append({"id": case["id"], "expected_kind": case["expected_kind"], "kind": kind, "category": profile["category"], "facts": count, "wrong_facts": wrong, "profile": profile})
    summary = {"kind_correct": correct, "cases": len(cases), "kind_accuracy": correct / len(cases), "facts_emitted": total, "wrong_facts": wrong_count, "fact_precision": (total - wrong_count) / total if total else None}
    result = {"labels_frozen_commit": FROZEN_COMMIT, "labels_sha256": hashlib.sha256(frozen).hexdigest(), "detector_sha256": hashlib.sha256((ROOT / "crates/hive-core/src/project_profile.rs").read_bytes()).hexdigest(), "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "summary": summary, "cases": rows}
    (work / "results.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(summary, ensure_ascii=False))


if __name__ == "__main__":
    main()
