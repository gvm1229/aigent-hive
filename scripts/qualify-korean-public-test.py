#!/usr/bin/env python3
"""Qualify the Korean core and pack lifecycle from one installed public binary."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any


def sha256(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def require_regular(path: Path, label: str) -> Path:
    resolved = path.resolve()
    if not resolved.is_file() or resolved.is_symlink():
        raise ValueError(f"{label} must be a regular non-symlink file: {resolved}")
    return resolved


def run_json(
    hive: Path,
    arguments: list[str],
    *,
    cwd: Path,
    expected_exit: int = 0,
) -> dict[str, Any]:
    completed = subprocess.run(
        [str(hive), *arguments, "--output", "json"],
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
        timeout=30,
    )
    if completed.returncode != expected_exit:
        raise RuntimeError(
            f"unexpected exit {completed.returncode}, expected {expected_exit}: "
            f"{' '.join(arguments)}\n{completed.stderr}"
        )
    try:
        result = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"command returned invalid JSON: {' '.join(arguments)}") from error
    if result.get("exit_code") != expected_exit:
        raise RuntimeError(f"result exit code disagrees for {' '.join(arguments)}")
    return result


def write_pack_version(source: Path, destination: Path, version: str) -> None:
    shutil.copytree(source, destination)
    rules_path = destination / "rules.json"
    rules = json.loads(rules_path.read_text(encoding="utf-8"))
    rules["pack_version"] = version
    rules["transform_version"] = int(rules["transform_version"]) + 1
    rules_path.write_text(
        json.dumps(rules, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    manifest_path = destination / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["pack_version"] = version
    manifest["transform_version"] = rules["transform_version"]
    manifest["rules_digest"] = sha256(rules_path)
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def qualify(
    hive: Path,
    pack: Path,
    corpus: Path,
    work_root: Path,
    expected_version: str,
) -> dict[str, Any]:
    hive = require_regular(hive, "Hive binary")
    pack = pack.resolve()
    corpus = require_regular(corpus, "Korean gold corpus")
    if not pack.is_dir() or pack.is_symlink():
        raise ValueError("language pack must be a regular directory")
    version = subprocess.run(
        [str(hive), "--version"],
        cwd=work_root,
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    ).stdout.strip()
    if version != expected_version:
        raise RuntimeError(f"unexpected public binary version: {version}")

    gold = json.loads(corpus.read_text(encoding="utf-8"))
    if gold.get("schema_version") != 1:
        raise ValueError("unsupported Korean corpus schema")
    with tempfile.TemporaryDirectory(prefix="korean-public-", dir=work_root) as directory:
        root = Path(directory)
        inspected = 0
        for case in gold["cases"]:
            input_path = root / f"inspect-{case['id']}.md"
            input_path.write_text(case["text"], encoding="utf-8")
            result = run_json(
                hive,
                ["korean", "inspect", "--profile", case["profile"], "--input", str(input_path)],
                cwd=root,
            )
            actual = {finding["rule_id"] for finding in result["data"]["findings"]}
            if actual != set(case["expected_rule_ids"]):
                raise RuntimeError(f"Korean inspect mismatch: {case['id']}")
            inspected += 1

        verified = 0
        rejected = 0
        for case in gold["verification_cases"]:
            before = root / f"verify-{case['id']}-before.md"
            after = root / f"verify-{case['id']}-after.md"
            before.write_text(case["before"], encoding="utf-8")
            after.write_text(case["after"], encoding="utf-8")
            expected_exit = 0 if case["accepted"] else 5
            result = run_json(
                hive,
                [
                    "korean", "verify", "--profile", case["profile"],
                    "--before", str(before), "--after", str(after),
                ],
                cwd=root,
                expected_exit=expected_exit,
            )
            if bool(result["data"]["accepted"]) != bool(case["accepted"]):
                raise RuntimeError(f"Korean verify mismatch: {case['id']}")
            verified += int(bool(case["accepted"]))
            rejected += int(not bool(case["accepted"]))

        controls = root / "controls.md"
        sanitized = root / "sanitized.md"
        controls.write_text("가\u200b나\u202e다", encoding="utf-8")
        sanitize_result = run_json(
            hive,
            ["korean", "sanitize", "--input", str(controls), "--output-file", str(sanitized)],
            cwd=root,
        )
        if sanitized.read_text(encoding="utf-8") != "가나다":
            raise RuntimeError("Korean sanitize did not remove bounded controls")
        if sanitize_result["data"]["watermark_claim"]:
            raise RuntimeError("Korean sanitize made a prohibited watermark claim")

        check = run_json(hive, ["korean", "pack", "check"], cwd=root)
        first = root / "pack-2.3.2"
        shutil.copytree(pack, first)
        target = root / "consumer"
        target.mkdir()
        preview = run_json(
            hive,
            ["korean", "pack", "preview", "--target", str(target), "--candidate", str(first)],
            cwd=root,
        )
        consent = preview["data"]["consent_digest"]
        run_json(
            hive,
            [
                "korean", "pack", "activate", "--target", str(target),
                "--candidate", str(first), "--consent-digest", consent,
            ],
            cwd=root,
            expected_exit=2,
        )
        activated = run_json(
            hive,
            [
                "korean", "pack", "activate", "--target", str(target),
                "--candidate", str(first), "--consent-digest", consent, "--confirm-pack",
            ],
            cwd=root,
        )
        second = root / "pack-2.3.3"
        write_pack_version(pack, second, "2.3.3")
        second_preview = run_json(
            hive,
            ["korean", "pack", "preview", "--target", str(target), "--candidate", str(second)],
            cwd=root,
        )
        run_json(
            hive,
            [
                "korean", "pack", "activate", "--target", str(target),
                "--candidate", str(second),
                "--consent-digest", second_preview["data"]["consent_digest"], "--confirm-pack",
            ],
            cwd=root,
        )
        rollback = run_json(
            hive,
            ["korean", "pack", "rollback", "--target", str(target)],
            cwd=root,
        )
        if rollback["data"]["pack_version"] != "2.3.2":
            raise RuntimeError("Korean pack rollback did not restore 2.3.2")

        return {
            "schema_version": 1,
            "status": "passed",
            "binary_version": version,
            "corpus_digest": sha256(corpus),
            "pack_manifest_digest": sha256(pack / "manifest.json"),
            "inspect_cases": inspected,
            "accepted_verification_cases": verified,
            "rejected_verification_cases": rejected,
            "pack_check_current_version": check["data"]["current_version"],
            "pack_check_latest_version": check["data"]["latest_version"],
            "initial_activation": bool(activated["data"]["activated"]),
            "rollback_version": rollback["data"]["pack_version"],
            "provider_api_calls": 0,
            "api_keys_read": 0,
        }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hive", type=Path, required=True)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--work-root", type=Path, required=True)
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    arguments = parser.parse_args()
    if not arguments.work_root.is_dir() or arguments.work_root.is_symlink():
        raise ValueError("work root must be a regular directory")
    result = qualify(
        arguments.hive,
        arguments.pack,
        arguments.corpus,
        arguments.work_root.resolve(),
        arguments.expected_version,
    )
    rendered = json.dumps(result, ensure_ascii=True, indent=2, sort_keys=True) + "\n"
    arguments.receipt.parent.mkdir(parents=True, exist_ok=True)
    arguments.receipt.write_text(rendered, encoding="utf-8")
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
