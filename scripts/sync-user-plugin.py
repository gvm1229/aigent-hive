#!/usr/bin/env python3
"""Refresh checked-in host and Copier projections from canonical Hive sources."""

from __future__ import annotations

import fnmatch
import hashlib
import json
import re
import shutil
from pathlib import Path, PurePosixPath


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
ACTIVE_SKILLS_LEDGER = ROOT / "harness" / "template" / ".hive" / "config" / "active-skills.yml.jinja"
UPSTREAM = ROOT / "harness" / "upstream"
UPSTREAM_RESOURCES_RS = ROOT / "crates" / "hive-projection" / "src" / "upstream_resources.rs"


def sync_directories(
    source_root: Path,
    destination_root: Path,
    *,
    copy_companions: bool,
    copy_project_openai_metadata: bool,
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
            # Mode-specific guidance must accompany a router in every host projection.
            for companion in ("references", "scripts"):
                if (source / companion).is_dir():
                    shutil.copytree(source / companion, destination / companion)
        if copy_project_openai_metadata:
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
                metadata.replace(policy_false, policy_true),
                encoding="utf-8",
                newline="\n",
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


def sync_active_skill_digests() -> None:
    text = ACTIVE_SKILLS_LEDGER.read_text(encoding="utf-8")
    pattern = re.compile(
        r"(?m)^(?P<prefix>- name: (?P<name>[a-z0-9-]+)\n"
        r"  source_type: built-in\n"
        r"  content_digest: sha256:)[0-9a-f]{64}$"
    )
    seen: set[str] = set()

    def replace(match: re.Match[str]) -> str:
        name = match.group("name")
        source = SOURCE / name / "SKILL.md"
        if not source.is_file():
            raise ValueError(f"active Skill ledger names missing source: {name}")
        seen.add(name)
        content = source.read_bytes()
        if b"\r\n" in content:
            raise ValueError(f"canonical Skill must use LF line endings before digest sync: {name}")
        digest = hashlib.sha256(content).hexdigest()
        return f"{match.group('prefix')}{digest}"

    updated = pattern.sub(replace, text)
    if not seen:
        raise ValueError("active Skill ledger has no built-in entries")
    ACTIVE_SKILLS_LEDGER.write_text(updated, encoding="utf-8", newline="\n")


def upstream_projection(source: dict, skill: str) -> list[str]:
    """Return the vendored files one Skill carries: all, or the lock's per-Skill glob subset."""
    root = UPSTREAM / source["name"]
    files = sorted(path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file())
    patterns = source.get("projection", {}).get(skill)
    if patterns is None:
        return files
    return [name for name in files if any(fnmatch.fnmatchcase(name, pattern) for pattern in patterns)]


def project_upstream_sources() -> list[tuple[str, str]]:
    """Copy vendored upstream trees byte-for-byte into `references/upstream/` of each user Skill."""
    lock = json.loads((UPSTREAM / "UPSTREAM.lock").read_text(encoding="utf-8"))
    expected: dict[str, set[str]] = {}
    resources: list[tuple[str, str]] = []
    for source in lock["sources"]:
        for skill in source["used_by"]:
            skill_root = SOURCE / skill
            if not (skill_root / "SKILL.md").is_file():
                continue
            expected.setdefault(skill, set()).add(source["name"])
            destination = skill_root / "references" / "upstream" / source["name"]
            if destination.exists():
                shutil.rmtree(destination)
            for relative in upstream_projection(source, skill):
                target = destination.joinpath(*PurePosixPath(relative).parts)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes((UPSTREAM / source["name"]).joinpath(*PurePosixPath(relative).parts).read_bytes())
                resources.append((skill, f"references/upstream/{source['name']}/{relative}"))
    for skill_root in SOURCE.iterdir():
        upstream_root = skill_root / "references" / "upstream"
        if upstream_root.is_dir():
            for child in upstream_root.iterdir():
                if child.name not in expected.get(skill_root.name, set()):
                    shutil.rmtree(child)
            if not any(upstream_root.iterdir()):
                upstream_root.rmdir()
    return sorted(resources)


def write_upstream_resources(resources: list[tuple[str, str]]) -> None:
    lines = [
        "// Generated by scripts/sync-user-plugin.py from harness/upstream/UPSTREAM.lock. Do not edit.",
        "",
        "//! Vendored upstream files carried by built-in Skills.",
        "",
        "/// Vendored upstream files each built-in Skill carries, byte-for-byte.",
        "#[rustfmt::skip]",
        "pub const UPSTREAM_RESOURCES: &[(&str, &str, &[u8])] = &[",
    ]
    for skill, relative in resources:
        lines.append(f'    ("{skill}", "{relative}", include_bytes!("../../../harness/skills/{skill}/{relative}")),')
    lines += ["];", ""]
    UPSTREAM_RESOURCES_RS.write_text("\n".join(lines), encoding="utf-8", newline="\n")


def main() -> int:
    write_upstream_resources(project_upstream_sources())
    for (
        destination,
        copy_companions,
        copy_project_metadata,
        excluded_names,
    ) in (
        SKILL_DESTINATIONS
    ):
        sync_directories(
            SOURCE,
            destination,
            copy_companions=copy_companions,
            copy_project_openai_metadata=copy_project_metadata,
            excluded_names=excluded_names,
        )
    sync_files(DIRECTIVE_SOURCE, DIRECTIVE_DESTINATION)
    sync_active_skill_digests()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
