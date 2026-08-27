#!/usr/bin/env python3
"""Qualify the Korean core and pack lifecycle from one installed public binary."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
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
        timeout=30,
    )
    if completed.returncode != expected_exit:
        raise RuntimeError(
            f"unexpected exit {completed.returncode}, expected {expected_exit}: "
            f"{' '.join(arguments)}\n{completed.stderr.decode('utf-8', errors='replace')}"
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
    rules["rules"][0]["threshold"] = 1
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
    product_version: str,
    package_version: str,
    release_date: str,
) -> dict[str, Any]:
    hive = require_regular(hive, "Hive binary")
    pack = pack.resolve()
    corpus = require_regular(corpus, "Korean gold corpus")
    if not pack.is_dir() or pack.is_symlink():
        raise ValueError("language pack must be a regular directory")
    package_match = re.fullmatch(
        re.escape(product_version) + r"-test\.([1-9][0-9]*)",
        package_version,
    )
    if package_match is None or re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", release_date) is None:
        raise ValueError("expected public version fields are invalid")
    expected_version = (
        f"AIgent Hive v{product_version}-test #{package_match.group(1)} "
        f"· developer test build (released {release_date})"
    )
    version_probe = subprocess.run(
        [str(hive), "--version"],
        cwd=work_root,
        check=True,
        capture_output=True,
        timeout=30,
    )
    version_bytes = version_probe.stdout.rstrip(b"\r\n")
    expected_version_bytes = expected_version.encode("utf-8")
    if version_bytes != expected_version_bytes:
        raise RuntimeError(
            "unexpected public binary version bytes: "
            f"actual={version_bytes.hex()} expected={expected_version_bytes.hex()}"
        )
    version = version_bytes.decode("utf-8")

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
        active_input = target / "active-rules.md"
        active_input.write_text("분석을 통해 결과를 확인했습니다.", encoding="utf-8")
        inspection = run_json(hive, ["korean", "inspect", "--target", str(target),
            "--profile", "response", "--input", str(active_input)], cwd=root)
        if inspection["data"]["pack_version"] != "2.3.3" or "A-2" not in {
            item["rule_id"] for item in inspection["data"]["findings"]
        }:
            raise RuntimeError("activated Korean rules were not consumed by inspection")
        rollback = run_json(
            hive,
            ["korean", "pack", "rollback", "--target", str(target)],
            cwd=root,
        )
        if rollback["data"]["pack_version"] != "2.3.2":
            raise RuntimeError("Korean pack rollback did not restore 2.3.2")
        restored = run_json(hive, ["korean", "inspect", "--target", str(target),
            "--profile", "response", "--input", str(active_input)], cwd=root)
        if restored["data"]["pack_version"] != "2.3.2" or any(
            item["rule_id"] == "A-2" for item in restored["data"]["findings"]
        ):
            raise RuntimeError("Korean rollback did not restore actual inspection behavior")
        invalid = root / "invalid-pack"
        shutil.copytree(second, invalid)
        (invalid / "rules.json").write_bytes(b"{}")
        invalid_manifest = json.loads((invalid / "manifest.json").read_text("utf-8"))
        invalid_manifest["rules_digest"] = sha256(invalid / "rules.json")
        (invalid / "manifest.json").write_text(json.dumps(invalid_manifest), encoding="utf-8")
        run_json(hive, ["korean", "pack", "preview", "--target", str(target),
            "--candidate", str(invalid)], cwd=root, expected_exit=2)

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
            "active_rules_consumed": True,
            "rollback_behavior_restored": True,
            "invalid_rules_rejected": True,
            "provider_api_calls": 0,
            "api_keys_read": 0,
        }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hive", type=Path, required=True)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--work-root", type=Path, required=True)
    parser.add_argument("--product-version", required=True)
    parser.add_argument("--package-version", required=True)
    parser.add_argument("--release-date", required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    arguments = parser.parse_args()
    if not arguments.work_root.is_dir() or arguments.work_root.is_symlink():
        raise ValueError("work root must be a regular directory")
    result = qualify(
        arguments.hive,
        arguments.pack,
        arguments.corpus,
        arguments.work_root.resolve(),
        arguments.product_version,
        arguments.package_version,
        arguments.release_date,
    )
    rendered = json.dumps(result, ensure_ascii=True, indent=2, sort_keys=True) + "\n"
    arguments.receipt.parent.mkdir(parents=True, exist_ok=True)
    arguments.receipt.write_text(rendered, encoding="utf-8")
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
