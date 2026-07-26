#!/usr/bin/env python3
"""Refresh checked-in host and Copier projections from canonical Hive sources."""

from __future__ import annotations

import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "harness" / "skills"
SKILL_DESTINATIONS = (
    ROOT / "harness" / "plugins" / "aigent-hive" / "skills",
    ROOT / "harness" / "template" / ".agents" / "skills",
    ROOT / "harness" / "template" / ".claude" / "skills",
)
DIRECTIVE_SOURCE = ROOT / "harness" / "directives"
DIRECTIVE_DESTINATION = (
    ROOT / "harness" / "template" / ".agents" / "directives"
)


def sync_directories(
    source_root: Path,
    destination_root: Path,
    *,
    copy_companions: bool,
) -> None:
    destination_root.mkdir(parents=True, exist_ok=True)
    expected = {
        child.name
        for child in source_root.iterdir()
        if child.is_dir() and (child / "SKILL.md").is_file()
    }
    for child in destination_root.iterdir():
        if child.is_dir() and child.name not in expected:
            shutil.rmtree(child)
    for name in sorted(expected):
        source = source_root / name
        destination = destination_root / name
        if destination.exists():
            shutil.rmtree(destination)
        if copy_companions:
            shutil.copytree(source, destination)
        else:
            destination.mkdir()
            shutil.copy2(source / "SKILL.md", destination / "SKILL.md")


def sync_files(source_root: Path, destination_root: Path) -> None:
    destination_root.mkdir(parents=True, exist_ok=True)
    expected = {
        child.name for child in source_root.iterdir() if child.is_file()
    }
    for child in destination_root.iterdir():
        if child.is_file() and child.name not in expected:
            child.unlink()
    for name in sorted(expected):
        shutil.copy2(source_root / name, destination_root / name)


def main() -> int:
    for index, destination in enumerate(SKILL_DESTINATIONS):
        sync_directories(
            SOURCE,
            destination,
            copy_companions=index == 0,
        )
    sync_files(DIRECTIVE_SOURCE, DIRECTIVE_DESTINATION)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
