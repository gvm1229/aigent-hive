#!/usr/bin/env python3
"""Verify public stable releases against the immutable Skill compatibility ledger."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

import yaml


def stable_version(value: str) -> bool:
    parts = value.split(".")
    return (
        len(parts) == 3
        and all(part.isdigit() and (part == "0" or not part.startswith("0")) for part in parts)
    )


def version_key(value: str) -> tuple[int, int, int]:
    return tuple(int(part) for part in value.split("."))  # type: ignore[return-value]


def published_versions(npm_payload: Any, github_payload: Any) -> set[str]:
    if not isinstance(npm_payload, list) or not all(isinstance(item, str) for item in npm_payload):
        raise ValueError("npm versions must be a string array")
    versions = {item for item in npm_payload if stable_version(item)}
    if not isinstance(github_payload, list):
        raise ValueError("GitHub releases must be an array")
    for release in github_payload:
        if not isinstance(release, dict):
            raise ValueError("GitHub release entries must be objects")
        tag = release.get("tagName")
        prerelease = release.get("isPrerelease")
        if not isinstance(tag, str) or not isinstance(prerelease, bool):
            raise ValueError("GitHub release entry is malformed")
        version = tag.removeprefix("v")
        if not prerelease and stable_version(version):
            versions.add(version)
    return {version for version in versions if version_key(version) >= (0, 8, 0)}


def current_skill_snapshot(historical_path: Path) -> list[dict[str, Any]]:
    skills_root = historical_path.parent
    catalog = yaml.safe_load((skills_root / "catalog.yml").read_text(encoding="utf-8"))
    entries = catalog.get("skills") if isinstance(catalog, dict) else None
    if not isinstance(entries, list):
        raise ValueError("current Skill catalog is malformed")
    snapshot: list[dict[str, Any]] = []
    for entry in entries:
        if not isinstance(entry, dict) or entry.get("availability") != "implemented":
            continue
        name = entry.get("name")
        side_effect = entry.get("side_effect_class")
        capabilities = entry.get("capabilities")
        if (
            not isinstance(name, str)
            or not isinstance(side_effect, str)
            or not isinstance(capabilities, list)
            or not all(isinstance(item, str) for item in capabilities)
        ):
            raise ValueError("current Skill catalog entry is malformed")
        skill_path = skills_root / name / "SKILL.md"
        digest = hashlib.sha256(skill_path.read_bytes()).hexdigest()
        snapshot.append(
            {
                "name": name,
                "content_digest": f"sha256:{digest}",
                "side_effect_class": side_effect,
                "capabilities": capabilities,
            }
        )
    if not snapshot:
        raise ValueError("current Skill snapshot is empty")
    return snapshot


def verify(
    ledger_path: Path,
    historical_path: Path,
    target_version: str,
    npm_payload: Any,
    github_payload: Any,
) -> dict[str, Any]:
    if not stable_version(target_version):
        raise ValueError("target version must be stable semver")
    ledger = yaml.safe_load(ledger_path.read_text(encoding="utf-8"))
    historical = yaml.safe_load(historical_path.read_text(encoding="utf-8"))
    if not isinstance(ledger, dict) or ledger.get("schema_version") != 1:
        raise ValueError("stable Skill ledger schema is unsupported")
    entries = ledger.get("stable_releases")
    if not isinstance(entries, list) or not entries:
        raise ValueError("stable Skill ledger is empty")
    historical_releases = historical.get("releases") if isinstance(historical, dict) else None
    if not isinstance(historical_releases, list):
        raise ValueError("historical Skill registry is malformed")
    historical_by_version = {
        release["version"]: release
        for release in historical_releases
        if isinstance(release, dict) and isinstance(release.get("version"), str)
    }
    current_skills = current_skill_snapshot(historical_path)

    versions: list[str] = []
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != {
            "version",
            "compatibility_epoch",
            "transition_proof",
        }:
            raise ValueError("stable Skill ledger entry has an invalid field set")
        version = entry["version"]
        epoch = entry["compatibility_epoch"]
        proof = entry["transition_proof"]
        if not isinstance(version, str) or not isinstance(epoch, str):
            raise ValueError("stable Skill ledger versions must be strings")
        if proof not in {"changed", "no-change"}:
            raise ValueError("stable Skill transition proof is invalid")
        known_versions = set(historical_by_version) | {target_version}
        if version not in known_versions or epoch not in known_versions:
            raise ValueError("stable Skill ledger references an unknown historical release")
        if proof == "changed" and epoch != version:
            raise ValueError("changed stable release must start its own compatibility epoch")
        if proof == "no-change":
            version_skills = (
                current_skills
                if version == target_version and version not in historical_by_version
                else historical_by_version[version].get("skills")
            )
            epoch_skills = (
                current_skills
                if epoch == target_version and epoch not in historical_by_version
                else historical_by_version[epoch].get("skills")
            )
            if version_skills != epoch_skills:
                raise ValueError("no-change epoch has different Skill bytes")
        versions.append(version)
    if versions != sorted(set(versions), key=version_key):
        raise ValueError("stable Skill ledger versions must be sorted and unique")

    expected = published_versions(npm_payload, github_payload) | {target_version}
    if set(versions) != expected:
        raise ValueError(
            "stable Skill ledger differs from published stable union plus target: "
            f"ledger={versions}, expected={sorted(expected, key=version_key)}"
        )
    return {
        "schema_version": 1,
        "status": "success",
        "target_version": target_version,
        "stable_versions": versions,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--historical", type=Path, required=True)
    parser.add_argument("--target-version", required=True)
    parser.add_argument("--npm-versions", type=Path, required=True)
    parser.add_argument("--github-releases", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = verify(
            args.ledger,
            args.historical,
            args.target_version,
            json.loads(args.npm_versions.read_text(encoding="utf-8")),
            json.loads(args.github_releases.read_text(encoding="utf-8")),
        )
    except (OSError, ValueError, json.JSONDecodeError, yaml.YAMLError) as error:
        print(f"stable-skill-ledger: {error}")
        return 1
    args.output.write_text(json.dumps(result, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
