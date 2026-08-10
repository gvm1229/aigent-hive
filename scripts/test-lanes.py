#!/usr/bin/env python3
"""Run the complete, named Python conformance inventory."""

from __future__ import annotations

import argparse
import subprocess
import sys
import time
import tomllib
from pathlib import Path
from typing import Any, Sequence


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "tests" / "conformance" / "lanes.toml"
TEST_ROOT = ROOT / "tests" / "conformance"


class LaneError(RuntimeError):
    """Raised when the executable conformance inventory is inconsistent."""


def load_manifest() -> dict[str, dict[str, Any]]:
    try:
        parsed = tomllib.loads(MANIFEST.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise LaneError(f"cannot read conformance lane manifest: {error}") from error
    lanes = parsed.get("lanes")
    if not isinstance(lanes, dict) or not lanes:
        raise LaneError("conformance lane manifest must define at least one lane")
    return lanes


def expected_modules() -> set[str]:
    return {
        f"tests.conformance.{path.stem}"
        for path in TEST_ROOT.glob("test_*.py")
        if path.is_file()
    }


def validate_manifest(lanes: dict[str, dict[str, Any]]) -> None:
    claimed: list[str] = []
    for name, lane in lanes.items():
        if not isinstance(lane, dict):
            raise LaneError(f"lane '{name}' must be a table")
        for field in ("owner", "contract", "release_gate", "modules"):
            if field not in lane:
                raise LaneError(f"lane '{name}' is missing '{field}'")
        modules = lane["modules"]
        if not isinstance(modules, list) or not modules or not all(
            isinstance(module, str) for module in modules
        ):
            raise LaneError(f"lane '{name}' must contain one or more module names")
        claimed.extend(modules)
    duplicate = sorted({module for module in claimed if claimed.count(module) > 1})
    if duplicate:
        raise LaneError("modules assigned to multiple lanes: " + ", ".join(duplicate))
    expected = expected_modules()
    actual = set(claimed)
    missing = sorted(expected - actual)
    unknown = sorted(actual - expected)
    if missing or unknown:
        details: list[str] = []
        if missing:
            details.append("unassigned: " + ", ".join(missing))
        if unknown:
            details.append("missing files: " + ", ".join(unknown))
        raise LaneError("; ".join(details))


def run_lane(name: str, lane: dict[str, Any]) -> None:
    started = time.monotonic()
    command = [sys.executable, "-m", "unittest", *lane["modules"], "-v"]
    print(f"+ lane={name} owner={lane['owner']} release_gate={lane['release_gate']}")
    try:
        subprocess.run(command, cwd=ROOT, check=True)
    except subprocess.CalledProcessError as error:
        raise LaneError(f"lane '{name}' failed with exit code {error.returncode}") from error
    elapsed = time.monotonic() - started
    print(f"+ lane={name} elapsed_seconds={elapsed:.2f}")


def parser() -> argparse.ArgumentParser:
    command = argparse.ArgumentParser(description=__doc__)
    group = command.add_mutually_exclusive_group(required=True)
    group.add_argument("--all", action="store_true", help="run every conformance lane")
    group.add_argument("--lane", choices=(), help="run one named conformance lane")
    group.add_argument("--list", action="store_true", help="list the validated lane inventory")
    return command


def main(arguments: Sequence[str] | None = None) -> int:
    lanes = load_manifest()
    command = parser()
    command._option_string_actions["--lane"].choices = tuple(lanes)
    parsed = command.parse_args(arguments)
    try:
        validate_manifest(lanes)
        if parsed.list:
            for name, lane in lanes.items():
                print(f"{name}: {len(lane['modules'])} modules; owner={lane['owner']}")
            return 0
        selected = lanes.items() if parsed.all else [(parsed.lane, lanes[parsed.lane])]
        for name, lane in selected:
            run_lane(name, lane)
    except LaneError as error:
        print(f"test-lanes: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
