#!/usr/bin/env python3
"""Qualify every public stable predecessor using authentic old-CLI project fixtures."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tomllib
import yaml
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("project_coverage", ROOT / "scripts/check-project-base-coverage.py")
coverage = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coverage)


def snapshot(target: Path) -> dict[str, str]:
    return {p.relative_to(target).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in target.rglob("*") if p.is_file()
            and not p.relative_to(target).as_posix().startswith((".hive/backups/", ".hive/runtime/"))}


def qualify(hive: Path, work: Path, target_version: str, failure_tests: bool) -> dict:
    versions = coverage.required_sources(target_version)
    if not versions:
        raise ValueError("no public stable predecessors found; fetch full release tags")
    rows = []
    work.mkdir(parents=True, exist_ok=True)
    for version in versions:
        fixture = ROOT / "tests/fixtures/project-predecessors" / version / "public-stable"
        manifest = json.loads((fixture / "manifest.json").read_bytes())
        archive = fixture / manifest["archive"]
        if manifest.get("schema_version") != 1 or manifest["product_version"] != version or hashlib.sha256(archive.read_bytes()).hexdigest() != manifest["sha256"]:
            raise ValueError(f"unauthenticated predecessor fixture: {version}")
        target = work / version
        target.mkdir()  # Never reuse a mutated fixture as a successful baseline.
        with ZipFile(archive) as z:
            assert len(z.infolist()) == manifest["file_count"], (version, "fixture file count")
            for item in z.infolist():
                name = Path(item.filename)
                if name.is_absolute() or ".." in name.parts or "\\" in item.filename:
                    raise ValueError("unsafe fixture path")
            z.extractall(target)
        agents = target / "AGENTS.md"
        prefix = b"# Team-owned guidance\r\nPreserve this section.\r\n"
        suffix = b"\r\nTeam-owned ending.\r\n"
        note = b"\nTeam note inside Hive block: prefer concise change summaries.\n"
        agents.write_bytes(prefix + agents.read_bytes().replace(b"<!-- AIGENT-HIVE:END -->", note + b"<!-- AIGENT-HIVE:END -->") + suffix)
        (target / "team-owned.txt").write_bytes(b"unchanged team content\r\n")
        before = snapshot(target)
        preferences = tomllib.loads((target / ".hive/config/harness.toml").read_text(encoding="utf-8"))
        assert preferences["harness_version"] == version
        assert json.loads((target / ".hive/config/project-base.json").read_bytes())["product_version"] == version

        def invoke(mode: str, env: dict | None = None, expected: int = 0):
            environment = {k: v for k, v in os.environ.items() if not k.startswith("HIVE_PROJECT_UPGRADE_")}
            environment.update(env or {})
            p = subprocess.run([str(hive), "project", "upgrade", "--target", str(target), mode, "--output", "json"],
                               capture_output=True, text=True, encoding="utf-8", env=environment)
            if p.returncode != expected:
                raise AssertionError((version, mode, expected, p.returncode, p.stdout, p.stderr))
            return json.loads(p.stdout) if p.stdout.strip() else None

        invoke("--scan")
        preview = invoke("--dry-run")
        assert snapshot(target) == before, (version, "preview mutated active files")
        assert preview["changed_paths"], (version, "empty predecessor upgrade")
        if failure_tests:
            # A normal error must roll back every active file, not just two samples.
            environment = {"HIVE_PROJECT_UPGRADE_FAIL_AFTER": "1"}
            p = subprocess.run([str(hive), "project", "upgrade", "--target", str(target), "--apply", "--output", "json"],
                               capture_output=True, env={**os.environ, **environment})
            assert p.returncode != 0, (version, "failure injection unavailable")
            assert snapshot(target) == before, (version, "incomplete rollback")
            invoke("--apply", {"HIVE_PROJECT_UPGRADE_INTERRUPT_AFTER": "1"}, expected=86)
            recovery = invoke("--recover")
            assert recovery["code"] == "hive.project-upgrade-recovered"
            assert snapshot(target) == before, (version, "interrupted transaction did not restore active files")
        applied = invoke("--apply")
        validated = invoke("--validate")
        assert validated["code"] == "hive.project-upgrade-current" and not validated["changed_paths"]
        after = snapshot(target)
        assert all(after.get(name) == digest for name, digest in before.items() if name not in applied["changed_paths"])
        content = agents.read_bytes()
        assert content.startswith(prefix) and content.endswith(suffix) and note in content
        updated = tomllib.loads((target / ".hive/config/harness.toml").read_text(encoding="utf-8"))
        assert updated["harness_version"] == target_version, (version, "wrong incoming binary version")
        for key, value in preferences.items():
            if key == "selected_project_skills":
                ledger = yaml.safe_load((ROOT / "harness/skills/retired-names.yml").read_text(encoding="utf-8"))
                expected = sorted({ledger["retired_names"].get(name, name) for name in value})
                assert updated[key] == expected, (version, "undeclared skill selection change")
                continue
            if key not in ("harness_version", "source_release_version"):
                assert updated[key] == value, (version, "preference changed", key)
        assert not invoke("--apply")["changed_paths"] and snapshot(target) == after
        rows.append({"source_version": version, "fixture_sha256": manifest["sha256"],
                     "scan_preview_apply_validate_idempotence": "passed", "local_and_foreign_preservation": "passed",
                     "rollback_and_interrupted_recovery": "passed" if failure_tests else "not-run: release binary has no failure injection"})
    return {"schema_version": 1, "target_version": target_version, "binary_sha256": hashlib.sha256(hive.read_bytes()).hexdigest(),
            "host_os": os.name, "required_sources": versions, "results": rows}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hive", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--target-version", required=True)
    parser.add_argument("--failure-tests", action="store_true")
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    result = qualify(args.hive.resolve(), args.work.resolve(), args.target_version, args.failure_tests)
    args.receipt.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"versions": result["required_sources"], "receipt": str(args.receipt)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
