#!/usr/bin/env python3
"""Prepare a deterministic local integrity bundle from one release candidate."""

from __future__ import annotations

import argparse
from datetime import date
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[1]
VERSION = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
SHA = re.compile(r"^[0-9a-f]{40}$")


def canonical_bytes(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    ).encode()


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as stream:
        return json.load(stream)


def write_json(path: Path, value: object) -> None:
    path.write_bytes(canonical_bytes(value))


def native_names(product_version: str) -> list[str]:
    return [
        f"aigent-hive-{product_version}-aarch64-apple-darwin.tar.gz",
        f"aigent-hive-{product_version}-aarch64-unknown-linux-musl.tar.gz",
        f"aigent-hive-{product_version}-x86_64-apple-darwin.tar.gz",
        f"aigent-hive-{product_version}-x86_64-pc-windows-msvc.zip",
        f"aigent-hive-{product_version}-x86_64-unknown-linux-musl.tar.gz",
    ]


def npm_names(package_version: str) -> list[str]:
    return [
        f"aigent-hive-darwin-arm64-{package_version}.tgz",
        f"aigent-hive-darwin-x64-{package_version}.tgz",
        f"aigent-hive-linux-arm64-{package_version}.tgz",
        f"aigent-hive-linux-x64-{package_version}.tgz",
        f"aigent-hive-win32-x64-{package_version}.tgz",
        f"aigent-hive-{package_version}.tgz",
    ]


def verify_candidate(args: argparse.Namespace) -> None:
    if not VERSION.fullmatch(args.product_version):
        raise ValueError("product version is invalid")
    if args.package_version != args.product_version:
        raise ValueError("stable integrity bundle requires matching product/package versions")
    if not SHA.fullmatch(args.candidate_sha):
        raise ValueError("candidate SHA must be 40 lowercase hexadecimal characters")
    if not args.candidate_run_id.isdigit() or args.candidate_run_id.startswith("0"):
        raise ValueError("candidate run id must be a positive decimal integer")
    if args.candidate_ref != "refs/heads/main":
        raise ValueError("stable integrity bundle requires refs/heads/main")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", args.repository):
        raise ValueError("repository must be an owner/name pair")

    candidate = load_json(args.dist / "release-candidate.json")
    expected = {
        "schema_version": 1,
        "channel": "stable",
        "product_version": args.product_version,
        "package_version": args.package_version,
        "ref": args.candidate_ref,
        "sha": args.candidate_sha,
    }
    if not isinstance(candidate, dict) or any(
        candidate.get(key) != value for key, value in expected.items()
    ):
        raise ValueError("release-candidate.json differs from the selected candidate")
    release_date = str(candidate.get("release_date", ""))
    try:
        date.fromisoformat(release_date)
    except ValueError as error:
        raise ValueError("release-candidate.json has no valid release date") from error


def require_candidate_files(dist: Path, names: list[str]) -> None:
    for name in names:
        artifact = dist / name
        if not artifact.is_file():
            raise ValueError(f"candidate artifact is missing: {name}")
        sidecar = dist / f"{name}.sha256"
        if name.endswith((".tar.gz", ".zip")):
            expected = f"{digest(artifact)}  {name}\n"
            if not sidecar.is_file() or sidecar.read_text(encoding="ascii") != expected:
                raise ValueError(f"checksum sidecar differs from candidate bytes: {name}")


def extract_installers(umbrella: Path, targets: Path) -> None:
    with tarfile.open(umbrella, "r:gz") as archive:
        for name in ("install.sh", "install.ps1", "install.cmd"):
            member = archive.getmember(f"package/{name}")
            if not member.isfile() or member.size > 1024 * 1024:
                raise ValueError(f"npm umbrella has no bounded installer: {name}")
            stream = archive.extractfile(member)
            if stream is None:
                raise ValueError(f"npm umbrella installer is unreadable: {name}")
            payload = stream.read(1024 * 1024 + 1)
            if len(payload) != member.size:
                raise ValueError(f"npm umbrella installer length changed: {name}")
            (targets / name).write_bytes(payload)


def prepare(args: argparse.Namespace) -> None:
    verify_candidate(args)
    names = native_names(args.product_version) + npm_names(args.package_version)
    require_candidate_files(args.dist, names)
    source = ROOT / "harness" / "release" / args.product_version
    migration = load_json(source / "migration-table.json")
    inventory = load_json(source / "release-surface-inventory.json")
    if not isinstance(migration, dict) or migration.get("target_version") != args.product_version:
        raise ValueError("canonical migration table targets another version")
    if not isinstance(inventory, dict) or inventory.get("product_version") != args.product_version:
        raise ValueError("canonical release surface targets another version")
    if args.output.exists():
        raise ValueError("integrity bundle output already exists")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix="release-integrity-", dir=args.output.parent))
    try:
        targets = temporary / "targets"
        targets.mkdir()
        for name in names:
            shutil.copyfile(args.dist / name, targets / name)
        extract_installers(args.dist / f"aigent-hive-{args.package_version}.tgz", targets)
        write_json(targets / "migration-table.json", migration)
        write_json(targets / "release-surface-inventory.json", inventory)

        artifacts = [
            {
                "length": path.stat().st_size,
                "path": f"targets/{path.name}",
                "sha256": digest(path),
            }
            for path in sorted(targets.iterdir(), key=lambda item: item.name)
        ]
        manifest = {
            "artifacts": artifacts,
            "classification": "feature",
            "license": "Apache-2.0",
            "migration_table_digest": (
                f"sha256:{digest(targets / 'migration-table.json')}"
            ),
            "minimum_supported_harness_version": "0.8.0",
            "minimum_updater_version": "0.9.0",
            "product": "aigent-hive",
            "release_sequence": 9,
            "release_version": args.product_version,
            "schema_version": 1,
            "source": {
                "commit": args.candidate_sha,
                "repository": f"https://github.com/{args.repository}",
                "tag": f"v{args.product_version}",
            },
            "surface_inventory_digest": (
                f"sha256:{digest(targets / 'release-surface-inventory.json')}"
            ),
        }
        write_json(temporary / "bundle-manifest.json", manifest)
        os.replace(temporary, args.output)
    except Exception:
        shutil.rmtree(temporary, ignore_errors=True)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--product-version", required=True)
    parser.add_argument("--package-version", required=True)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--candidate-run-id", required=True)
    parser.add_argument("--candidate-ref", required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--dist", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


if __name__ == "__main__":
    try:
        prepare(parse_args())
    except (OSError, ValueError, json.JSONDecodeError, tarfile.TarError) as error:
        raise SystemExit(f"release integrity preparation failed: {error}") from error
