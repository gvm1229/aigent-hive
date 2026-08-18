#!/usr/bin/env python3
"""Validate frozen full project bases for a declared same-major migration range."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FULL_BASE_MINIMUM = (0, 9, 1)


def version(value: str) -> tuple[int, int, int]:
    parts = value.split(".")
    if len(parts) != 3 or any(not part.isdigit() for part in parts):
        raise ValueError(f"invalid exact version: {value}")
    return tuple(int(part) for part in parts)  # type: ignore[return-value]


def projected_paths(release: str) -> dict[str, Path]:
    names = subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", f"v{release}", "harness/template"],
        cwd=ROOT,
        text=True,
    ).splitlines()
    base = ROOT / "harness/project-bases" / release
    paths: dict[str, Path] = {}
    for source in names:
        suffix = source.removeprefix("harness/template/")
        if suffix == "AGENTS.md.jinja":
            destination = base / "AGENTS.md.template"
        elif suffix.startswith(".agents/directives/"):
            destination = base / "directives" / suffix.removeprefix(".agents/directives/")
        elif suffix.startswith(".agents/skills/"):
            destination = base / "skills" / suffix.removeprefix(".agents/skills/")
        else:
            continue
        paths[source] = destination
    return paths


def frozen_base_digest(release: str) -> str:
    paths = projected_paths(release)
    if not paths:
        raise ValueError(f"empty frozen project-base inventory: {release}")
    actual = {path for path in (ROOT / "harness/project-bases" / release).rglob("*") if path.is_file()}
    if actual != set(paths.values()):
        raise ValueError(f"frozen project-base inventory differs from v{release}")
    digest = hashlib.sha256()
    for source, destination in sorted(paths.items()):
        expected = subprocess.check_output(["git", "show", f"v{release}:{source}"], cwd=ROOT)
        actual_bytes = destination.read_bytes()
        if actual_bytes != expected:
            raise ValueError(f"frozen project-base bytes differ from v{release}: {source}")
        digest.update(source.encode("utf-8"))
        digest.update(b"\0")
        digest.update(actual_bytes)
        digest.update(b"\0")
    return f"sha256:{digest.hexdigest()}"


def tagged_sources() -> list[str]:
    tags = subprocess.check_output(["git", "tag", "--list", "v*"], cwd=ROOT, text=True).splitlines()
    releases = []
    for tag in tags:
        try:
            version(tag.removeprefix("v"))
        except ValueError:
            continue
        releases.append(tag.removeprefix("v"))
    return sorted(releases, key=version)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--migration-table", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()

    table = json.loads(arguments.migration_table.read_text(encoding="utf-8"))
    if table.get("schema_version") != 1 or not isinstance(table.get("routes"), list):
        raise ValueError("invalid migration table")
    target = version(table["target_version"])
    sources = tagged_sources()
    coverage: list[dict[str, object]] = []
    for route in table["routes"]:
        if route.get("kind") != "same-major":
            continue
        lower = version(route["from_min"])
        upper = version(route["from_max"])
        if lower < FULL_BASE_MINIMUM:
            raise ValueError("same-major migration declares a source without a full project base")
        selected = [
            release
            for release in sources
            if version(release)[0] == target[0] and lower <= version(release) <= upper
        ]
        if not selected:
            raise ValueError(f"same-major migration has no tagged source: {route.get('route_id')}")
        coverage.append(
            {
                "route_id": route["route_id"],
                "sources": [
                    {"version": release, "base_digest": frozen_base_digest(release)}
                    for release in selected
                ],
            }
        )
    if not coverage:
        raise ValueError("migration table has no same-major project coverage")
    report = {
        "schema_version": 1,
        "target_version": table["target_version"],
        "coverage": coverage,
    }
    canonical = json.dumps(report, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode("utf-8")
    report["coverage_digest"] = f"sha256:{hashlib.sha256(canonical).hexdigest()}"
    arguments.output.write_text(
        json.dumps(report, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
