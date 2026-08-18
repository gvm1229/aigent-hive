#!/usr/bin/env python3
"""Run isolated public-artifact acceptance for Hive project or user updates."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import Any


SUPPORT_FILES = (
    ".hive/setup-answers.yml",
    ".hive/config/harness.toml",
    ".hive/config/capability-resolution.yml",
    ".hive/config/active-skills.yml",
    ".hive/config/approved-skills.yml",
    ".hive/config/knowledge-scope.yml",
    ".hive/config/project-overrides.json",
    ".hive/config/role-seeds.yml",
    ".hive/config/project-base.json",
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def regular_file(path: Path, label: str) -> None:
    if not path.is_file() or path.is_symlink():
        raise ValueError(f"{label} is not a regular file: {path}")


def copy_file(source: Path, destination: Path) -> None:
    regular_file(source, "acceptance source")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)


def run(command: list[str], environment: dict[str, str], cwd: Path) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=cwd,
        env=environment,
        check=False,
        text=True,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed ({completed.returncode}): {' '.join(command)}\n{completed.stderr}"
        )
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"command did not return one JSON object: {' '.join(command)}") from error


def hive_version(hive: Path, environment: dict[str, str], cwd: Path) -> str:
    completed = subprocess.run(
        [str(hive), "--version"],
        cwd=cwd,
        env=environment,
        check=False,
        text=True,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"Hive version probe failed: {completed.stderr}")
    version = completed.stdout.strip()
    if not version:
        raise RuntimeError("Hive version probe returned no version text")
    return version


def fixture_manifest(root: Path, paths: list[str]) -> dict[str, str]:
    result: dict[str, str] = {}
    for relative in sorted(paths):
        path = root / relative
        regular_file(path, "fixture file")
        result[relative] = sha256(path)
    return result


def manifest_digest(manifest: dict[str, str]) -> str:
    payload = json.dumps(manifest, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def copy_project_fixture(source: Path, destination: Path) -> tuple[dict[str, str], list[str]]:
    ledger_path = source / ".hive/config/project-base.json"
    regular_file(ledger_path, "project base ledger")
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    if ledger.get("schema_version") != 1 or ledger.get("product_version") != "0.9.2":
        raise ValueError("project base ledger is not the expected 0.9.2 ledger")
    files = ledger.get("files")
    if not isinstance(files, list) or len(files) != 48:
        raise ValueError("project base ledger does not contain 48 expected files")
    projection_paths = [entry.get("path") for entry in files if isinstance(entry, dict)]
    if any(not isinstance(path, str) for path in projection_paths):
        raise ValueError("project base ledger contains an invalid path")
    copied = [*SUPPORT_FILES, *projection_paths]
    if len(set(copied)) != len(copied):
        raise ValueError("fixture copy set has duplicate paths")
    for relative in copied:
        copy_file(source / relative, destination / relative)
    sentinel = destination / "FOREIGN-ACCEPTANCE-SENTINEL.txt"
    sentinel.write_bytes(b"foreign acceptance bytes\r\n")
    return fixture_manifest(source, copied), copied


def scoped_environment(test_root: Path) -> dict[str, str]:
    environment = os.environ.copy()
    codex_root = test_root / ".codex"
    codex_root.mkdir(exist_ok=True)
    environment["CODEX_HOME"] = str(codex_root)
    if os.name == "nt":
        environment["USERPROFILE"] = str(test_root)
    else:
        environment["HOME"] = str(test_root)
    return environment


def verify_project(source: Path, hive: Path, test_root: Path) -> dict[str, Any]:
    fixture = test_root / "project"
    source_manifest, copied = copy_project_fixture(source, fixture)
    environment = scoped_environment(test_root)
    results = []
    for mode in ("--scan", "--dry-run", "--apply", "--validate"):
        results.append(
            run(
                [str(hive), "project", "upgrade", "--target", str(fixture), mode, "--output", "json"],
                environment,
                fixture,
            )
        )
    sentinel = fixture / "FOREIGN-ACCEPTANCE-SENTINEL.txt"
    if sentinel.read_bytes() != b"foreign acceptance bytes\r\n":
        raise RuntimeError("project upgrade changed the foreign acceptance sentinel")
    if fixture_manifest(source, copied) != source_manifest:
        raise RuntimeError("project acceptance changed the source project")

    local_fixture = test_root / "project-local"
    _, local_copied = copy_project_fixture(source, local_fixture)
    local_marker = b"\n<!-- acceptance local addition -->\n"
    with (local_fixture / "AGENTS.md").open("ab") as marker_file:
        marker_file.write(local_marker)
    for mode in ("--scan", "--dry-run", "--apply", "--validate"):
        result = run(
            [str(hive), "project", "upgrade", "--target", str(local_fixture), mode, "--output", "json"],
            environment,
            local_fixture,
        )
        results.append(result)
    if not (local_fixture / "AGENTS.md").read_bytes().endswith(local_marker):
        raise RuntimeError("project upgrade did not preserve the local AGENTS addition")
    if (local_fixture / "FOREIGN-ACCEPTANCE-SENTINEL.txt").read_bytes() != b"foreign acceptance bytes\r\n":
        raise RuntimeError("project upgrade changed the local fixture foreign sentinel")

    tampered_fixture = test_root / "project-tampered"
    _, tampered_copied = copy_project_fixture(source, tampered_fixture)
    tampered_ledger = tampered_fixture / ".hive/config/project-base.json"
    tampered = json.loads(tampered_ledger.read_text(encoding="utf-8"))
    tampered["files"][0]["content"] = "tampered acceptance ledger\n"
    tampered_ledger.write_text(json.dumps(tampered, ensure_ascii=True, separators=(",", ":")), encoding="utf-8")
    before = fixture_manifest(tampered_fixture, tampered_copied)
    failed = subprocess.run(
        [str(hive), "project", "upgrade", "--target", str(tampered_fixture), "--scan", "--output", "json"],
        cwd=tampered_fixture,
        env=environment,
        check=False,
        text=True,
        capture_output=True,
    )
    if failed.returncode == 0 or fixture_manifest(tampered_fixture, tampered_copied) != before:
        raise RuntimeError("tampered project base did not fail without mutation")

    return {
        "mode": "project",
        "source_file_count": len(copied),
        "source_manifest_digest": manifest_digest(source_manifest),
        "fixture_manifest_digest": manifest_digest(fixture_manifest(fixture, copied)),
        "local_fixture_manifest_digest": manifest_digest(fixture_manifest(local_fixture, local_copied)),
        "tampered_failure_exit_code": failed.returncode,
        "result_codes": [result["code"] for result in results],
    }


def verify_user(hive: Path, test_root: Path, prepare_only: bool) -> dict[str, Any]:
    environment = scoped_environment(test_root)
    codex_program = "codex.cmd" if os.name == "nt" else "codex"
    codex = subprocess.run(
        [codex_program, "plugin", "marketplace", "list", "--json"],
        cwd=test_root,
        env=environment,
        check=False,
        text=True,
        capture_output=True,
    )
    if codex.returncode != 0:
        raise RuntimeError(f"isolated Codex probe failed: {codex.stderr}")
    json.loads(codex.stdout)
    answers = test_root / "answers.yml"
    answers.write_text(
        """schema_version: 1
interface_language: en
wiki:
  enabled: true
  language: en
  backend: markdown
profile:
  contexts:
    - non-developer
persona:
  id: strict
selected_hosts:
  - codex
skills:
  mode: all
update_check:
  enabled: true
usage_guard:
  enabled: true
  stop_remaining_percent: 20
  codexbar_fallback_enabled: false
  discord:
    enabled: false
    request_privacy: summary
    message_fields:
      - remaining-usage
      - project
      - request
      - progress
      - host
      - resume
  project_overrides: {}
""",
        encoding="utf-8",
    )
    commands = [
        [str(hive), "setup", "--scope", "user", "--quick-answers", str(answers), "--user-root", str(test_root), "--apply", "--output", "json"],
        [str(hive), "install", "--scope", "user", "--host", "codex", "--user-root", str(test_root), "--apply", "--output", "json"],
        [str(hive), "install", "--scope", "user", "--host", "codex", "--user-root", str(test_root), "--validate", "--output", "json"],
        [str(hive), "update", "--check", "--user-root", str(test_root), "--output", "json"],
        [str(hive), "update", "--channel", "test", "--user-root", str(test_root), "--confirm"],
        [str(hive), "install", "--scope", "user", "--host", "codex", "--user-root", str(test_root), "--validate", "--output", "json"],
    ]
    command_count = 3 if prepare_only else 4
    results: list[dict[str, Any]] = []
    for command in commands[:command_count]:
        results.append(run(command, environment, test_root))
    if prepare_only:
        return {
            "mode": "user-prepare",
            "result_codes": [result["code"] for result in results],
        }
    initial_version = hive_version(hive, environment, test_root)
    completed = subprocess.run(
        commands[4], cwd=test_root, env=environment, check=False, text=True, capture_output=True
    )
    if completed.returncode != 0:
        raise RuntimeError(f"test-channel update failed: {completed.stderr}")
    results.append({"command": "update", "code": completed.returncode, "stderr": completed.stderr})
    for _ in range(60):
        final_version = hive_version(hive, environment, test_root)
        if final_version != initial_version:
            break
        time.sleep(0.5)
    else:
        raise RuntimeError("test-channel update handoff did not activate a new Hive version")
    for _ in range(60):
        validation = subprocess.run(
            commands[5], cwd=test_root, env=environment, check=False, text=True, capture_output=True
        )
        if validation.returncode == 0:
            results.append({"command": "validate", "code": validation.returncode})
            break
        time.sleep(0.5)
    else:
        raise RuntimeError("test-channel update handoff did not complete its user projection validation")
    return {
        "mode": "user",
        "initial_version": initial_version,
        "final_version": final_version,
        "result_codes": [result["code"] for result in results],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=("project", "user"), required=True)
    parser.add_argument("--hive", type=Path, required=True)
    parser.add_argument("--work-root", type=Path, required=True)
    parser.add_argument("--source-project", type=Path)
    parser.add_argument("--prepare-only", action="store_true")
    arguments = parser.parse_args()
    hive = arguments.hive.resolve()
    regular_file(hive, "Hive executable")
    work_root = arguments.work_root.resolve()
    if not work_root.is_dir():
        raise ValueError("work root must be an existing directory")
    test_root = work_root / f"public-hive-acceptance-{uuid.uuid4().hex}"
    test_root.mkdir()
    if arguments.mode == "project":
        if arguments.source_project is None:
            raise ValueError("project mode requires --source-project")
        source = arguments.source_project.resolve()
        if not source.is_dir() or (source / "hive-source.json").exists():
            raise ValueError("source project is not an eligible consumer root")
        result = verify_project(source, hive, test_root)
    else:
        result = verify_user(hive, test_root, arguments.prepare_only)
    result["test_root"] = str(test_root)
    print(json.dumps(result, ensure_ascii=True, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
