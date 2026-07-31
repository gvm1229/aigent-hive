#!/usr/bin/env python3
"""Refresh checked-in host and Copier projections from canonical Hive sources."""

from __future__ import annotations

import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "harness" / "skills"
SKILL_DESTINATIONS = (
    (
        ROOT / "harness" / "plugins" / "aigent-hive" / "skills",
        True,
        False,
        frozenset(),
    ),
    (
        ROOT / "harness" / "template" / ".agents" / "skills",
        False,
        True,
        frozenset({"setup-hive"}),
    ),
    (
        ROOT / "harness" / "template" / ".claude" / "skills",
        False,
        False,
        frozenset({"setup-hive"}),
    ),
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
    copy_explicit_openai_metadata: bool,
    excluded_names: frozenset[str],
) -> None:
    destination_root.mkdir(parents=True, exist_ok=True)
    expected = {
        child.name
        for child in source_root.iterdir()
        if (
            child.is_dir()
            and child.name not in excluded_names
            and (child / "SKILL.md").is_file()
        )
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
        if copy_explicit_openai_metadata:
            source_metadata = source / "agents" / "openai.yaml"
            destination_metadata = destination / "agents" / "openai.yaml"
            destination_metadata.parent.mkdir()
            metadata = source_metadata.read_text(encoding="utf-8")
            policy_true = "  allow_implicit_invocation: true"
            policy_false = "  allow_implicit_invocation: false"
            if policy_true not in metadata and policy_false not in metadata:
                raise ValueError(
                    f"missing Codex invocation policy: {source_metadata}"
                )
            destination_metadata.write_text(
                metadata.replace(policy_true, policy_false),
                encoding="utf-8",
            )


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
    for (
        destination,
        copy_companions,
        copy_explicit_metadata,
        excluded_names,
    ) in (
        SKILL_DESTINATIONS
    ):
        sync_directories(
            SOURCE,
            destination,
            copy_companions=copy_companions,
            copy_explicit_openai_metadata=copy_explicit_metadata,
            excluded_names=excluded_names,
        )
    sync_files(DIRECTIVE_SOURCE, DIRECTIVE_DESTINATION)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
