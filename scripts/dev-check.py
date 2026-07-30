#!/usr/bin/env python3
"""Run local Hive verification without changing the caller's environment."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Sequence


ROOT = Path(__file__).resolve().parents[1]
REQUIREMENTS = ROOT / "requirements-conformance.txt"
WINDOWS_SOURCE_GUARD_TESTS = (
    "tests.conformance.test_source_usage_guard.SourceUsageGuardTests."
    "test_write_json_skips_unavailable_fchmod",
    "tests.conformance.test_source_usage_guard.SourceUsageGuardTests."
    "test_write_json_closes_descriptor_before_failed_write_cleanup",
    "tests.conformance.test_source_usage_guard.SourceUsageGuardTests."
    "test_windows_watcher_lease_read_skips_locked_first_byte",
    "tests.conformance.test_source_usage_guard.SourceUsageGuardTests."
    "test_gate_allows_clean_clone_without_omx_state",
    "tests.conformance.test_source_usage_guard.SourceUsageGuardTests."
    "test_disabled_gate_does_not_initialize_quota_sensor",
)


class DevCheckError(RuntimeError):
    """An actionable local verification setup error."""


def executable_name(name: str) -> str:
    return f"{name}.exe" if os.name == "nt" else name


def resolve_tool(name: str, *, rust_tool: bool = False) -> Path:
    resolved = shutil.which(name)
    if resolved:
        return Path(resolved)

    if rust_tool:
        executable = executable_name(name)
        candidates = [Path.home() / ".cargo" / "bin" / executable]
        candidates.extend(
            sorted(
                (Path.home() / ".rustup" / "toolchains").glob(
                    f"stable-*/bin/{executable}"
                )
            )
        )
        for candidate in candidates:
            if candidate.is_file():
                return candidate

    hint = (
        "Install Rust with rustup or expose its stable toolchain on PATH."
        if rust_tool
        else f"Install {name} and add it to PATH."
    )
    raise DevCheckError(f"Required tool '{name}' was not found. {hint}")


def tool_environment(*tools: Path) -> dict[str, str]:
    environment = os.environ.copy()
    directories = list(dict.fromkeys(str(tool.parent) for tool in tools))
    inherited_path = environment.get("PATH")
    if inherited_path:
        directories.append(inherited_path)
    environment["PATH"] = os.pathsep.join(directories)
    return environment


def run(command: Sequence[str], *, environment: dict[str, str]) -> None:
    print("+", " ".join(command), flush=True)
    try:
        subprocess.run(command, cwd=ROOT, env=environment, check=True)
    except subprocess.CalledProcessError as error:
        raise DevCheckError(
            f"Verification command failed with exit code {error.returncode}: "
            f"{' '.join(command)}"
        ) from error


def run_rust(arguments: Sequence[str]) -> None:
    cargo = resolve_tool("cargo", rust_tool=True)
    rustc = resolve_tool("rustc", rust_tool=True)
    environment = tool_environment(cargo, rustc)
    environment["RUSTC"] = str(rustc)

    if arguments:
        run([str(cargo), *arguments], environment=environment)
        return

    for command in (
        [str(cargo), "fmt", "--all", "--check"],
        [
            str(cargo),
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        [
            str(cargo),
            "test",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--locked",
        ],
    ):
        run(command, environment=environment)


def run_python(arguments: Sequence[str]) -> None:
    uv = resolve_tool("uv")
    cargo = resolve_tool("cargo", rust_tool=True)
    rustc = resolve_tool("rustc", rust_tool=True)
    environment = tool_environment(uv, cargo, rustc)
    environment["RUSTC"] = str(rustc)
    environment.pop("HIVE_WINDOWS_SOURCE_USAGE_GUARD_SUBSET", None)
    unittest_arguments = list(arguments) or [
        "discover",
        "-s",
        "tests/conformance",
        "-t",
        ".",
        "-p",
        "test_*.py",
        "-v",
    ]
    prefix = [
        str(uv),
        "run",
        "--python",
        "3.13",
        "--isolated",
        "--no-project",
        "--with-requirements",
        str(REQUIREMENTS),
        "python",
        "-m",
        "unittest",
    ]
    if os.name == "nt" and not arguments:
        run(
            [*prefix, *WINDOWS_SOURCE_GUARD_TESTS, "-v"],
            environment=environment,
        )
        environment["HIVE_WINDOWS_SOURCE_USAGE_GUARD_SUBSET"] = "skip"
    run([*prefix, *unittest_arguments], environment=environment)


def parser() -> argparse.ArgumentParser:
    command_parser = argparse.ArgumentParser(
        description="Run reproducible local verification."
    )
    command_parser.add_argument("mode", choices=("rust", "python", "pre-push"))
    command_parser.add_argument(
        "arguments",
        nargs=argparse.REMAINDER,
        help="targeted cargo or unittest arguments for rust/python mode",
    )
    return command_parser


def main(arguments: Sequence[str] | None = None) -> int:
    parsed = parser().parse_args(arguments)
    passthrough = parsed.arguments
    if passthrough[:1] == ["--"]:
        passthrough = passthrough[1:]

    try:
        if parsed.mode == "rust":
            run_rust(passthrough)
        elif parsed.mode == "python":
            run_python(passthrough)
        else:
            if passthrough:
                raise DevCheckError(
                    "pre-push does not accept targeted arguments; "
                    "use rust or python mode."
                )
            run_rust(())
            run_python(())
    except DevCheckError as error:
        print(f"dev-check: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
