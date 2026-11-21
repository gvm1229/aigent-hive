#!/usr/bin/env python3
"""Collect read-only local host qualification inputs as JSON; make no verdict."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def invoke(command: list[str], *, cwd: Path) -> dict[str, object]:
    try:
        result = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)
        return {"command": command, "returncode": result.returncode,
                "stdout": result.stdout, "stderr": result.stderr}
    except OSError as error:
        return {"command": command, "returncode": None, "stdout": "", "stderr": str(error)}


def project_inputs(user_root: Path, scan_root: Path) -> dict[str, object]:
    registry = user_root / ".hive/config/projects.yml"
    registered: list[object] = []
    parse_error = None
    if registry.is_file():
        try:
            import yaml
        except ImportError as error:
            parse_error = str(error)
        else:
            try:
                content = yaml.safe_load(registry.read_text(encoding="utf-8")) or {}
                registered = content.get("projects", []) if isinstance(content, dict) else []
            except (OSError, UnicodeError, yaml.YAMLError, ValueError, TypeError) as error:
                parse_error = str(error)
    disk = sorted(path.resolve().as_posix() for path in scan_root.iterdir()
                  if path.is_dir() and (path / ".hive").is_dir()) if scan_root.is_dir() else []
    return {"registry_path": str(registry), "registry_exists": registry.is_file(),
            "registered_projects": registered, "registry_parse_error": parse_error,
            "scan_root": str(scan_root), "disk_projects": disk}


def skill_names(user_root: Path) -> dict[str, list[str]]:
    result: dict[str, list[str]] = {}
    locations = {
        "claude": (user_root / ".claude/skills",),
        "codex": (user_root / ".agents/skills",),
        "gemini": (user_root / ".hive/marketplaces/antigravity/plugins/aigent-hive/skills",),
    }
    for host, roots in locations.items():
        result[host] = sorted({entry.name for root in roots if root.is_dir() for entry in root.iterdir()
                               if entry.is_dir() and (entry / "SKILL.md").is_file()})
    return result


def collect(args: argparse.Namespace) -> dict[str, object]:
    target = args.target.resolve()
    user_root = args.user_root.resolve()
    command = shlex.split(args.hive) if args.hive else []
    if command and not (Path(command[0]).is_file() or shutil.which(command[0])):
        command = []
    if not command and not args.hive:
        executable = shutil.which("hive")
        command = [executable] if executable else []
    if command and os.name == "nt":
        executable_path = Path(command[0])
        powershell_script = executable_path if executable_path.suffix.lower() == ".ps1" else Path(str(executable_path) + ".ps1")
        if powershell_script.is_file():
            command = ["powershell", "-File", str(powershell_script), *command[1:]]
    report: dict[str, object] = {
        "schema_version": 1,
        "scope": {"target": str(target), "user_root": str(user_root), "host": args.host},
        "inputs": {"installed_version": None, "user_install_validation": None,
                   "usage_enforcement": None, "skill_projection_names": skill_names(user_root),
                   "hook_preview": None, "flow_start_preview": {
                       "supported": False,
                       "reason": "installed CLI has no flow-start preview option"},
                   "project_registry_comparison": project_inputs(user_root, target)},
    }
    if not command:
        report["hive_available"] = False
        return report
    report["hive_available"] = True
    report["inputs"]["installed_version"] = invoke(command + ["--version"], cwd=target)
    report["inputs"]["user_install_validation"] = invoke(
        command + ["install", "--scope", "user", "--host", args.host, "--validate",
                   "--user-root", str(user_root), "--output", "json"], cwd=target)
    work_root = ROOT / "tests/work"
    work_root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="qualify-local-hosts-", dir=work_root) as temporary:
        scratch = Path(temporary)
        scratch_target = scratch / "target"
        scratch_user = scratch / "user-root"
        scratch_target.mkdir()
        scratch_user.mkdir()
        report["inputs"]["usage_enforcement"] = invoke(
            command + ["usage", "enforce", "--target", str(scratch_target), "--host", args.host,
                       "--session-id", "local-qualification", "--process-id", str(os.getpid()),
                       "--user-root", str(scratch_user), "--output", "json"], cwd=target)
    hook_host = "gemini" if args.host == "antigravity" else args.host
    report["inputs"]["hook_preview"] = invoke(
        command + ["msg", "hooks", "preview", "--host", hook_host, "--role", "athena",
                   "--target", str(target), "--user-root", str(user_root), "--output", "json"], cwd=target)
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", type=Path, default=Path.cwd())
    parser.add_argument("--user-root", type=Path, default=Path.home())
    parser.add_argument("--host", choices=("codex", "claude", "antigravity"), default="codex")
    parser.add_argument("--hive", help="Hive command; supports a quoted executable and arguments")
    args = parser.parse_args()
    print(json.dumps(collect(args), ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
