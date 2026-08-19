#!/usr/bin/env python3
"""Run the complete, named Python conformance inventory."""

from __future__ import annotations

import argparse
import json
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
        "tests.conformance."
        + ".".join(path.relative_to(TEST_ROOT).with_suffix("").parts)
        for path in TEST_ROOT.rglob("test_*.py")
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


def run_lane(name: str, lane: dict[str, Any], *, module_timings: bool) -> dict[str, Any]:
    started = time.monotonic()
    print(f"+ lane={name} owner={lane['owner']} release_gate={lane['release_gate']}")
    timings: list[dict[str, object]] = []
    groups = [[module] for module in lane["modules"]] if module_timings else [lane["modules"]]
    for modules in groups:
        module_started = time.monotonic()
        command = [sys.executable, "-m", "unittest", *modules, "-v"]
        try:
            subprocess.run(command, cwd=ROOT, check=True)
        except subprocess.CalledProcessError as error:
            raise LaneError(f"lane '{name}' failed with exit code {error.returncode}") from error
        if module_timings:
            timings.append(
                {
                    "module": modules[0],
                    "elapsed_seconds": round(time.monotonic() - module_started, 3),
                }
            )
    elapsed = time.monotonic() - started
    print(f"+ lane={name} elapsed_seconds={elapsed:.2f}")
    return {
        "name": name,
        "owner": lane["owner"],
        "release_gate": lane["release_gate"],
        "elapsed_seconds": round(elapsed, 3),
        "modules": timings,
    }


def changed_paths(reference: str) -> list[str]:
    completed = subprocess.run(
        ["git", "diff", "--name-only", reference, "--"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode:
        raise LaneError(completed.stderr.strip() or f"cannot compare changed paths with {reference}")
    return [line.strip().replace("\\", "/") for line in completed.stdout.splitlines() if line.strip()]


def lanes_for_paths(paths: Sequence[str], available: Sequence[str]) -> list[str]:
    selected: set[str] = set()
    all_lanes = list(available)
    for path in paths:
        if path.startswith("tests/conformance/"):
            part = path.split("/", 3)[2]
            mapped = "contract" if part == "contracts" else part
            if mapped in all_lanes:
                selected.add(mapped)
                continue
        if path.endswith((".md", ".md.jinja")) or path.startswith("docs/"):
            selected.add("documentation")
        elif path.startswith(("crates/hive-update/", "harness/release/", "npm/")) or path.startswith(
            ("scripts/package-", "scripts/check-release", ".github/workflows/release")
        ):
            selected.update({"release", "security"})
        elif path.startswith(("crates/hive-wiki/", "schemas/knowledge-")):
            selected.update({"contract", "integration", "security"})
        elif path.startswith(("crates/", "harness/", "schemas/", "tests/fixtures/")):
            selected.update({"contract", "integration", "security"})
        elif path.startswith(("scripts/", ".github/", "Cargo.", "requirements-")):
            return all_lanes
        else:
            return all_lanes
    return [name for name in all_lanes if name in selected] or all_lanes


def parser() -> argparse.ArgumentParser:
    command = argparse.ArgumentParser(description=__doc__)
    group = command.add_mutually_exclusive_group(required=True)
    group.add_argument("--all", action="store_true", help="run every conformance lane")
    group.add_argument("--lane", choices=(), help="run one named conformance lane")
    group.add_argument("--list", action="store_true", help="list the validated lane inventory")
    command.add_argument("--changed-from", help="with --all, run only lanes affected since this Git ref")
    command.add_argument("--timing-json", type=Path, help="write lane and module timing evidence")
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
        if parsed.changed_from and not parsed.all:
            raise LaneError("--changed-from requires --all")
        names = list(lanes)
        if parsed.changed_from:
            names = lanes_for_paths(changed_paths(parsed.changed_from), names)
            print("+ changed_lanes=" + ",".join(names))
        elif not parsed.all:
            names = [parsed.lane]
        reports = [
            run_lane(name, lanes[name], module_timings=parsed.timing_json is not None)
            for name in names
        ]
        if parsed.timing_json:
            rendered = json.dumps(
                {"schema_version": 1, "lanes": reports}, ensure_ascii=False, indent=2
            ) + "\n"
            parsed.timing_json.parent.mkdir(parents=True, exist_ok=True)
            parsed.timing_json.write_text(rendered, encoding="utf-8")
    except LaneError as error:
        print(f"test-lanes: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
