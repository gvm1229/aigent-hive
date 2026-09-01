#!/usr/bin/env python3
"""Enforce the stable-only documentation shown to ordinary end users."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "docs/public-stable-release.json"
SEMVER = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
MARKER = re.compile(
    r"<!-- AIGENT-HIVE:PUBLIC-STABLE version=(\d+\.\d+\.\d+) release-date=(\d{4}-\d{2}-\d{2}) -->"
)
HEADING = re.compile(r"^(#{1,6})\s+(.+?)\s*$", re.MULTILINE)
PUBLIC_PRERELEASE = re.compile(
    r"(?:\b\d+\.\d+\.\d+-(?:test|alpha|beta|rc)(?:\.\d+)?\b|"
    r"aigent-hive@test\b|--channel\s+test\b|/releases/tag/v[^\s)]+-(?:test|alpha|beta|rc))",
    re.IGNORECASE,
)
RELEASE_ID = re.compile(r"^- \[([A-Z][A-Z0-9-]+)\] ", re.MULTILINE)


@dataclass(frozen=True)
class Failure:
    code: str
    path: str
    detail: str


def anchor(value: str) -> str:
    lowered = value.casefold().replace("`", "")
    lowered = re.sub(r"[^\w가-힣 -]", "", lowered)
    return re.sub(r"\s+", "-", lowered.strip())


def add(failures: list[Failure], code: str, path: str, detail: str) -> None:
    failures.append(Failure(code, path, detail))


def load_manifest(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("manifest must be an object")
    return value


def public_text(root: Path, relative: str) -> str:
    return (root / relative).read_text(encoding="utf-8")


def validate_marker(
    failures: list[Failure], relative: str, text: str, version: str, release_date: str
) -> None:
    matches = MARKER.findall(text)
    if matches != [(version, release_date)]:
        add(failures, "stable-marker", relative, f"expected one marker for {version} {release_date}")


def validate_public_surface(
    failures: list[Failure], root: Path, relative: str, version: str, release_date: str
) -> None:
    text = public_text(root, relative)
    validate_marker(failures, relative, text, version, release_date)
    if PUBLIC_PRERELEASE.search(text):
        add(failures, "prerelease-exposure", relative, "numbered prerelease or test install path")
    if relative == "README.md":
        required = (
            f"badge/version-{version}-",
            f"Stable `{version}` is the current public release.",
            f"aigent-hive@{version}",
            f"AIgent Hive v{version} (released {release_date})",
            f"https://unpkg.com/aigent-hive@{version}/install.sh",
            f"https://unpkg.com/aigent-hive@{version}/install.ps1",
            f"https://unpkg.com/aigent-hive@{version}/install.cmd",
            f"## What changed in {version}",
        )
    elif relative == "docs/readme/README.ko.md":
        required = (
            f"badge/version-{version}-",
            f"현재 stable `{version}`",
            f"aigent-hive@{version}",
            f"AIgent Hive v{version} (released {release_date})",
            f"https://unpkg.com/aigent-hive@{version}/install.sh",
            f"https://unpkg.com/aigent-hive@{version}/install.ps1",
            f"https://unpkg.com/aigent-hive@{version}/install.cmd",
            f"## {version} 주요 변경",
        )
    elif relative == "docs/hive-install-guide.ko.html":
        required = (
            f"Stable <code>{version}</code>",
            f"AIgent Hive v{version} (released {release_date})",
            f"https://unpkg.com/aigent-hive@{version}/install.sh",
            f"https://unpkg.com/aigent-hive@{version}/install.ps1",
            f"https://unpkg.com/aigent-hive@{version}/install.cmd",
            f"Stable {version} · npm latest",
        )
    elif relative == "docs/overview/product.md":
        required = (
            f"Current public stable version: `{version}` (released {release_date})",
            f"[`{version}` release](../releases/{version}.md)",
        )
    elif relative == "docs/01-index.md":
        required = (f"[`{version}`](releases/{version}.md)",)
    else:
        required = ()
    for value in required:
        if value not in text:
            add(failures, "required-stable-text", relative, value)


def release_scope_ids(root: Path, relative: str) -> set[str]:
    text = public_text(root, relative)
    english = text.split("## 한국어", 1)[0]
    selected: list[str] = []
    for heading in ("### Scope", "### Compatibility"):
        try:
            body = english.split(heading, 1)[1]
        except IndexError as error:
            raise ValueError(f"release notes missing {heading}") from error
        body = re.split(r"^### ", body, maxsplit=1, flags=re.MULTILINE)[0]
        selected.extend(RELEASE_ID.findall(body))
    return set(selected)


def validate_coverage(
    failures: list[Failure], root: Path, manifest: dict[str, object]
) -> None:
    release_notes = manifest.get("release_notes")
    coverage = manifest.get("coverage")
    if not isinstance(release_notes, str) or not isinstance(coverage, dict):
        add(failures, "manifest-shape", "docs/public-stable-release.json", "release_notes or coverage")
        return
    expected = release_scope_ids(root, release_notes)
    if set(coverage) != expected:
        add(failures, "coverage-set", "docs/public-stable-release.json", f"expected {sorted(expected)}")
        return
    for identifier, entry in coverage.items():
        if not isinstance(entry, dict):
            add(failures, "coverage-entry", str(identifier), "object required")
            continue
        status = entry.get("status")
        if status == "documented":
            path = entry.get("path")
            expected_anchor = entry.get("anchor")
            if not isinstance(path, str) or not isinstance(expected_anchor, str):
                add(failures, "coverage-document", str(identifier), "path and anchor required")
                continue
            try:
                anchors = {anchor(match.group(2)) for match in HEADING.finditer(public_text(root, path))}
            except OSError:
                add(failures, "coverage-document", str(identifier), f"missing {path}")
                continue
            if expected_anchor not in anchors:
                add(failures, "coverage-anchor", str(identifier), f"{path}#{expected_anchor}")
        elif status in {"maintainer-only", "not-user-facing"}:
            if not isinstance(entry.get("reason"), str) or not entry["reason"].strip():
                add(failures, "coverage-reason", str(identifier), "nonempty reason required")
        else:
            add(failures, "coverage-status", str(identifier), str(status))


def registry_versions(path: Path) -> set[str]:
    values = {line.split("\t", 1)[1].strip() for line in path.read_text(encoding="utf-8").splitlines() if "\t" in line}
    if not values:
        raise ValueError("registry latest file has no package rows")
    return values


def check(arguments: argparse.Namespace) -> dict[str, object]:
    root = arguments.root.resolve()
    manifest_path = root / arguments.manifest
    manifest = load_manifest(manifest_path)
    failures: list[Failure] = []
    version = manifest.get("stable_version")
    release_date = manifest.get("release_date")
    surfaces = manifest.get("public_surfaces")
    if not isinstance(version, str) or not SEMVER.fullmatch(version):
        add(failures, "manifest-version", str(arguments.manifest), str(version))
        version = "0.0.0"
    if not isinstance(release_date, str) or not DATE.fullmatch(release_date):
        add(failures, "manifest-date", str(arguments.manifest), str(release_date))
        release_date = "1970-01-01"
    if not isinstance(surfaces, list) or any(not isinstance(item, str) for item in surfaces):
        add(failures, "manifest-surfaces", str(arguments.manifest), "string list required")
        surfaces = []
    for relative in surfaces:
        try:
            validate_public_surface(failures, root, relative, version, release_date)
        except OSError:
            add(failures, "missing-surface", relative, "cannot read")
    validate_coverage(failures, root, manifest)
    if arguments.channel == "stable":
        if arguments.product_version != version or arguments.release_date != release_date:
            add(failures, "stable-target", str(arguments.manifest), f"{arguments.product_version} {arguments.release_date}")
    if arguments.registry_latest_file:
        latest = registry_versions(root / arguments.registry_latest_file)
        if latest != {version}:
            add(failures, "registry-latest", arguments.registry_latest_file, repr(sorted(latest)))
    return {
        "schema_version": 1,
        "status": "success" if not failures else "error",
        "channel": arguments.channel,
        "stable_version": version,
        "release_date": release_date,
        "checked_surfaces": surfaces,
        "failures": [asdict(item) for item in failures],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--manifest", default="docs/public-stable-release.json")
    parser.add_argument("--channel", choices=("local", "test", "stable"), default="local")
    parser.add_argument("--product-version")
    parser.add_argument("--release-date")
    parser.add_argument("--registry-latest-file")
    parser.add_argument("--output", choices=("json",), default="json")
    arguments = parser.parse_args()
    if arguments.channel == "stable" and (not arguments.product_version or not arguments.release_date):
        parser.error("stable channel requires --product-version and --release-date")
    try:
        result = check(arguments)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        result = {"schema_version": 1, "status": "error", "failures": [{"code": "checker", "path": "", "detail": str(error)}]}
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))
    return 0 if result["status"] == "success" else 1


if __name__ == "__main__":
    raise SystemExit(main())
