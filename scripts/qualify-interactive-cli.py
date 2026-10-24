#!/usr/bin/env python3
"""Run bounded CLI inspections with a private fixture environment.

This is not an OS sandbox. Mutating lifecycle scenarios must run in an isolated OS.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


class IsolationError(ValueError):
    """An inspection would escape its explicit fixture scope."""


def beneath(path: Path, parent: Path) -> Path:
    # Resolve existing links as well as links in ancestors of new paths.
    resolved = path.resolve()
    if resolved == parent or not resolved.is_relative_to(parent):
        raise IsolationError("path must be a child of the owned fixture scope")
    return resolved


class Inspection:
    def __init__(self, fixture: Path, binary: Path, *, source: Path = ROOT):
        self.source = source.resolve()
        work = (self.source / "tests/work").resolve()
        if not work.is_relative_to(self.source):
            raise IsolationError("tests/work escapes the source workspace")
        self.fixture = beneath(fixture, work)
        self.binary = beneath(binary, self.source)
        if not self.binary.is_file():
            raise IsolationError("an explicit source or fixture binary is required")
        if self.binary.name not in {"hive", "hive.exe"}:
            raise IsolationError("expected an explicit Hive executable")
        self.user = beneath(self.fixture / "user", self.fixture)
        self.fake_bin = beneath(self.fixture / "fake-bin", self.fixture)

    def arguments(self, arguments: list[str]) -> list[str]:
        # An allowlist is safer than attempting to block every mutating subcommand.
        allowed = (
            ["--version"],
            ["setup", "--scope", "user", "--describe"],
            ["setup", "--scope", "user", "--show"],
        )
        if arguments not in allowed:
            raise IsolationError("only version, user setup describe and show are allowed on-host")
        if arguments == ["--version"]:
            return arguments
        if arguments[-1] == "--describe":
            # Catalog queries reject --user-root and use no installed settings.
            return [*arguments, "--output", "json"]
        return [*arguments, "--user-root", str(self.user), "--output", "json"]

    def environment(self) -> dict[str, str]:
        # No inherited PATH, home, npm prefix, host profile, proxy, or sensor overrides.
        directories = {
            "HOME": "user", "USERPROFILE": "user", "APPDATA": "appdata",
            "LOCALAPPDATA": "local-appdata", "CODEX_HOME": "codex",
            "CLAUDE_CONFIG_DIR": "claude", "GEMINI_CLI_HOME": "gemini",
            "XDG_CONFIG_HOME": "config", "XDG_CACHE_HOME": "cache",
            "XDG_DATA_HOME": "data", "NPM_CONFIG_PREFIX": "npm-prefix",
            "NPM_CONFIG_CACHE": "npm-cache", "TEMP": "tmp", "TMP": "tmp",
            "PATH": "fake-bin",
        }
        environment = {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"}
        for key, relative in directories.items():
            path = beneath(self.fixture / relative, self.fixture)
            path.mkdir(parents=True, exist_ok=True)
            environment[key] = str(path)
        # Even absolute Windows adapter resolution cannot reach the real scheduler.
        environment["SystemRoot"] = str(beneath(self.fixture / "no-system-tools", self.fixture))
        environment["WINDIR"] = environment["SystemRoot"]
        return environment

    def run(self, arguments: list[str], *, timeout: float = 30) -> subprocess.CompletedProcess:
        command = self.arguments(arguments)  # Validate before creating any fixture files.
        # Revalidate links immediately before constructing paths and executing.
        fresh = Inspection(self.fixture, self.binary, source=self.source)
        return subprocess.run(
            [str(fresh.binary), *command], cwd=fresh.fixture,
            env=fresh.environment(), stdin=subprocess.DEVNULL,
            capture_output=True, timeout=timeout, check=False,
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--scenario", choices=("version", "describe", "show"), required=True)
    options = parser.parse_args()
    arguments = {
        "version": ["--version"],
        "describe": ["setup", "--scope", "user", "--describe"],
        "show": ["setup", "--scope", "user", "--show"],
    }[options.scenario]
    try:
        result = Inspection(options.fixture, options.binary).run(arguments)
        sys.stdout.buffer.write(result.stdout)
        sys.stderr.buffer.write(result.stderr)
        return result.returncode
    except (IsolationError, OSError, subprocess.TimeoutExpired) as error:
        print(json.dumps({"status": "refused", "reason": str(error)}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
