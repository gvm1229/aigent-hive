#!/usr/bin/env python3
"""Qualify vector-search onboarding from one installed public Hive binary."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import tempfile
from pathlib import Path
from typing import Any


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


def data(result: dict[str, Any]) -> dict[str, Any]:
    if result.get("status") != "success" or not isinstance(result.get("data"), dict):
        raise RuntimeError(f"expected successful action result: {result}")
    return result["data"]


def feature(hive: Path, root: Path, *arguments: str, expected_exit: int = 0) -> dict[str, Any]:
    return run_json(
        hive,
        ["setup", "feature", *arguments, "--id", "vector-search", "--user-root", str(root)],
        cwd=root,
        expected_exit=expected_exit,
    )


def qualify(
    hive: Path,
    work_root: Path,
    product_version: str,
    package_version: str,
    release_date: str,
) -> dict[str, Any]:
    hive = require_regular(hive, "Hive binary")
    match = re.fullmatch(
        re.escape(product_version) + r"-test\.([1-9][0-9]*)", package_version
    )
    if match is None or re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", release_date) is None:
        raise ValueError("expected public version fields are invalid")
    expected_version = (
        f"AIgent Hive v{product_version}-test #{match.group(1)} "
        f"· developer test build (released {release_date})"
    ).encode("utf-8")
    version = subprocess.run(
        [str(hive), "--version"], cwd=work_root, check=True, capture_output=True, timeout=30
    ).stdout.rstrip(b"\r\n")
    if version != expected_version:
        raise RuntimeError("public binary version bytes differ from the selected test package")

    with tempfile.TemporaryDirectory(prefix="vector-onboarding-public-", dir=work_root) as directory:
        root = Path(directory)
        yes_root = root / "yes-root"
        yes_root.mkdir()
        claim = data(feature(hive, yes_root, "claim"))
        if claim.get("question_required") is not True or claim.get("answer") is not None:
            raise RuntimeError("fresh user root did not receive exactly one vector question")
        competing = data(feature(hive, yes_root, "claim"))
        if competing.get("question_required") is not False or competing.get("question_pending") is not True:
            raise RuntimeError("concurrent vector question claim was not suppressed")
        answered = data(feature(hive, yes_root, "answer", "--answer", "yes"))
        if answered.get("answer") != "yes" or not isinstance(answered.get("answered_at_unix"), int):
            raise RuntimeError("yes answer was not retained")
        prompt = data(feature(hive, yes_root, "prompt"))
        request_digest = prompt.get("setup_request_digest")
        prompt_text = prompt.get("prompt")
        if (
            prompt.get("scope_collection_ids") != ["user-root"]
            or not isinstance(request_digest, str)
            or not re.fullmatch(r"sha256:[0-9a-f]{64}", request_digest)
            or not isinstance(prompt_text, str)
            or request_digest not in prompt_text
            or "project-private" not in prompt_text
            or "confidential" not in prompt_text
        ):
            raise RuntimeError("yes prompt did not retain its fixed safe scope")
        answer_bytes = (yes_root / ".hive/config/user-feature-answers.yml").read_bytes()
        if b"session" in answer_bytes.lower() or b"codex" in answer_bytes.lower():
            raise RuntimeError("saved vector answer exposed a host session identity")

        no_root = root / "no-root"
        no_root.mkdir()
        data(feature(hive, no_root, "claim"))
        no = data(feature(hive, no_root, "answer", "--answer", "no"))
        if no.get("answer") != "no":
            raise RuntimeError("no answer was not retained")
        status = data(feature(hive, no_root, "status"))
        if status.get("answer") != "no" or status.get("question_pending") is not False:
            raise RuntimeError("no answer did not suppress repeat onboarding")
        rejected = feature(hive, no_root, "prompt", expected_exit=3)
        if rejected.get("status") != "conflict":
            raise RuntimeError("no answer unexpectedly prepared a vector setup prompt")
        return {
            "schema_version": 1,
            "product_version": product_version,
            "package_version": package_version,
            "release_date": release_date,
            "yes_answer": answered["answer"],
            "no_answer": status["answer"],
            "scope_collection_ids": prompt["scope_collection_ids"],
            "setup_request_digest": request_digest,
        }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hive", required=True, type=Path)
    parser.add_argument("--work-root", required=True, type=Path)
    parser.add_argument("--product-version", required=True)
    parser.add_argument("--package-version", required=True)
    parser.add_argument("--release-date", required=True)
    parser.add_argument("--receipt", required=True, type=Path)
    args = parser.parse_args()
    root = args.work_root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    receipt = args.receipt.resolve()
    receipt.parent.mkdir(parents=True, exist_ok=True)
    result = qualify(
        args.hive,
        root,
        args.product_version,
        args.package_version,
        args.release_date,
    )
    receipt.write_text(json.dumps(result, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
