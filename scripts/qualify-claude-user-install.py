#!/usr/bin/env python3
"""Qualify user installation with a real, isolated Claude CLI; no model calls."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def digest(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def qualify(binary: Path, claude: Path, work: Path) -> dict:
    if work.exists():
        raise ValueError("qualification requires a fresh work directory")
    if not work.resolve().is_relative_to((ROOT / "tests/work").resolve()):
        raise ValueError("qualification work must remain under source tests/work")
    work.mkdir(parents=True)
    expected = tomllib.loads((ROOT / "Cargo.toml").read_text("utf-8"))["workspace"]["package"]["version"]
    safe_names = {"PATH", "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT", "TEMP", "TMP", "LANG", "LC_ALL"}
    base = {key: value for key, value in os.environ.items() if key.upper() in safe_names}
    base.update(
        PATH=str(claude.parent) + os.pathsep + base.get("PATH", ""),
        CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC="1",
        DISABLE_AUTOUPDATER="1", DISABLE_TELEMETRY="1", DISABLE_ERROR_REPORTING="1",
    )
    report = {
        "schema_version": 1, "host": "codex", "os": platform.system(),
        "binary_digest": digest(binary), "claude_digest": digest(claude),
        "product_version": expected, "native_cli": True, "model_calls": 0,
        "scenarios": [], "status": "running",
    }
    for name in ("ordinary", "user space", "사용자 한글"):
        user = work / name
        user.mkdir()
        environment = dict(base, HOME=str(user), USERPROFILE=str(user),
                           CLAUDE_CONFIG_DIR=str(user / ".claude"),
                           APPDATA=str(user / "appdata"), LOCALAPPDATA=str(user / "localappdata"))
        guidance = user / ".claude/CLAUDE.md"
        guidance.parent.mkdir()
        foreign = b"# Existing user guidance\nPreserve these exact bytes.\n"
        guidance.write_bytes(foreign)
        settings = user / ".claude/settings.json"
        settings.write_text(json.dumps({"env": {"HIVE_QUALIFICATION_FOREIGN": "preserved"}}), encoding="utf-8")
        preserved = user / ".hive/knowledge/preserved.txt"
        preserved.parent.mkdir(parents=True)
        preserved.write_bytes(b"existing user knowledge\n")
        preference = user / ".hive/config/preserved.txt"
        preference.parent.mkdir()
        preference.write_bytes(b"existing user preference\n")
        retained = {path: path.read_bytes() for path in (preserved, preference)}

        def invoke(program: Path, arguments: list[str]) -> str:
            result = subprocess.run([str(program), *arguments], cwd=user, env=environment,
                                    stdin=subprocess.DEVNULL, capture_output=True, text=True,
                                    encoding="utf-8", errors="replace", timeout=120)
            if result.returncode:
                # Never publish raw host output; retain only bounded metadata.
                raise RuntimeError(f"qualification command {arguments[0]} failed: exit={result.returncode}; "
                                   f"stdout-digest={hashlib.sha256(result.stdout.encode()).hexdigest()}; "
                                   f"stderr-digest={hashlib.sha256(result.stderr.encode()).hexdigest()}")
            return result.stdout

        report["claude_version"] = invoke(claude, ["--version"]).strip()
        report["binary_version"] = invoke(binary, ["--version"]).strip()
        assert f"v{expected}" in report["binary_version"]
        row = {"name": name, "operations": []}
        for action, mode in (("install", "--dry-run"), ("install", "--apply"),
                             ("install", "--validate"), ("install", "--apply"),
                             ("update", "--apply"), ("install", "--validate")):
            value = json.loads(invoke(binary, [action, "--scope", "user", "--host", "claude",
                                             "--user-root", str(user), mode, "--output", "json"]))
            assert value["status"] == "success", value["code"]
            row["operations"].append({"action": action, "mode": mode, "code": value["code"]})
            assert foreign in guidance.read_bytes() if mode != "--dry-run" else guidance.read_bytes() == foreign
            for path, content in retained.items():
                assert path.read_bytes() == content
            assert json.loads(settings.read_text("utf-8"))["env"]["HIVE_QUALIFICATION_FOREIGN"] == "preserved"
        marketplace = json.loads(invoke(claude, ["plugin", "marketplace", "list", "--json"]))
        plugin = json.loads(invoke(claude, ["plugin", "list", "--json"]))
        assert any(entry["name"] == "aigent-hive" and entry["source"] == "directory" for entry in marketplace)
        assert any(entry["id"] == "aigent-hive@aigent-hive" and entry["version"] == expected
                   and entry["enabled"] and entry["scope"] == "user" for entry in plugin)
        assert not (user / ".hive/install-transactions/claude.json").exists()
        row["preserved"] = ["foreign-guidance", "foreign-settings", "knowledge", "preferences"]
        row["status"] = "passed"
        report["scenarios"].append(row)
    report["status"] = "passed"
    report["not_proven"] = ["interactive-skill-discovery", "question-tool-invocation", "natural-language-skill-invocation",
                            "UNC-and-long-path-support", "original-reporters-machine"]
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--claude", type=Path)
    parser.add_argument("--work-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    claude = args.claude or Path(shutil.which("claude") or "")
    report = qualify(args.binary.resolve(strict=True), claude.resolve(strict=True), args.work_root.resolve())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "scenarios": len(report["scenarios"]),
                      "claude_version": report["claude_version"]}, ensure_ascii=False))


if __name__ == "__main__":
    main()
