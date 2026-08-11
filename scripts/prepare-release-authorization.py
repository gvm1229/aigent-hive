#!/usr/bin/env python3
"""Prepare a deterministic, public-only TUF authorization request."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tempfile
from datetime import datetime


ROOT = Path(__file__).resolve().parents[1]
VERSION = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
SHA = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")


def canonical_bytes(value: object) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


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


def require_timestamp(value: str, label: str) -> datetime:
    if not value.endswith("Z"):
        raise ValueError(f"{label} must be an ISO-8601 UTC timestamp")
    try:
        return datetime.fromisoformat(value[:-1] + "+00:00")
    except ValueError as error:
        raise ValueError(f"{label} must be an ISO-8601 UTC timestamp") from error


def archive_names(product_version: str) -> list[str]:
    return [
        f"aigent-hive-{product_version}-aarch64-apple-darwin.tar.gz",
        f"aigent-hive-{product_version}-aarch64-unknown-linux-musl.tar.gz",
        f"aigent-hive-{product_version}-x86_64-apple-darwin.tar.gz",
        f"aigent-hive-{product_version}-x86_64-pc-windows-msvc.zip",
        f"aigent-hive-{product_version}-x86_64-unknown-linux-musl.tar.gz",
    ]


def verify_candidate(args: argparse.Namespace) -> dict[str, object]:
    if not VERSION.fullmatch(args.product_version) or args.package_version != args.product_version:
        raise ValueError("authorization requests require one exact stable product/package version")
    if not SHA.fullmatch(args.candidate_sha):
        raise ValueError("candidate SHA must be 40 lowercase hexadecimal characters")
    if not args.candidate_run_id.isdigit() or args.candidate_run_id.startswith("0"):
        raise ValueError("candidate run id must be a positive decimal integer")
    if args.candidate_ref != "refs/heads/main":
        raise ValueError("stable authorization requires refs/heads/main")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", args.repository):
        raise ValueError("repository must be an owner/name pair")
    started = require_timestamp(args.started_on, "started-on")
    finished = require_timestamp(args.finished_on, "finished-on")
    if started > finished:
        raise ValueError("candidate finish time precedes its start time")

    candidate_path = args.dist / "release-candidate.json"
    candidate = load_json(candidate_path)
    expected = {
        "schema_version": 1,
        "channel": "stable",
        "product_version": args.product_version,
        "package_version": args.package_version,
        "ref": args.candidate_ref,
        "sha": args.candidate_sha,
    }
    if not isinstance(candidate, dict) or any(candidate.get(key) != value for key, value in expected.items()):
        raise ValueError("release-candidate.json does not match the authorized stable candidate")
    if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", str(candidate.get("release_date", ""))):
        raise ValueError("release-candidate.json has no valid release date")
    return candidate


def verify_archives(dist: Path, names: list[str]) -> dict[str, str]:
    digests: dict[str, str] = {}
    for name in names:
        archive = dist / name
        sidecar = dist / f"{name}.sha256"
        if not archive.is_file() or not sidecar.is_file():
            raise ValueError(f"candidate artifact is missing: {name}")
        actual = digest(archive)
        expected_line = f"{actual}  {name}\n"
        if sidecar.read_text(encoding="ascii") != expected_line:
            raise ValueError(f"checksum sidecar differs from candidate bytes: {name}")
        digests[name] = actual
    return digests


def prepare(args: argparse.Namespace) -> None:
    verify_candidate(args)
    names = archive_names(args.product_version)
    archive_digests = verify_archives(args.dist, names)
    source = ROOT / "harness" / "release" / args.product_version
    migration = load_json(source / "migration-table.json")
    inventory = load_json(source / "release-surface-inventory.json")
    if not isinstance(migration, dict) or migration.get("target_version") != args.product_version:
        raise ValueError("canonical migration table targets another version")
    if not isinstance(inventory, dict) or inventory.get("product_version") != args.product_version:
        raise ValueError("canonical release surface targets another version")
    if args.output.exists():
        raise ValueError("authorization output already exists")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix="release-authorization-", dir=args.output.parent))
    try:
        targets = temporary / "targets"
        targets.mkdir()
        for name in names:
            shutil.copyfile(args.dist / name, targets / name)
        write_json(targets / "migration-table.json", migration)
        write_json(targets / "release-surface-inventory.json", inventory)

        platform_entries = []
        for name in names:
            if name.endswith("-apple-darwin.tar.gz"):
                platform_entries.append({
                    "artifact_digest": f"sha256:{archive_digests[name]}",
                    "artifact_path": f"targets/{name}",
                    "platform": "macos",
                    "scheme": "ad-hoc",
                    "signer": {"kind": "no-publisher", "value": ""},
                    "status": "cost-waived",
                })
            elif name.endswith("-pc-windows-msvc.zip"):
                platform_entries.append({
                    "artifact_digest": f"sha256:{archive_digests[name]}",
                    "artifact_path": f"targets/{name}",
                    "platform": "windows",
                    "scheme": "unsigned",
                    "signer": {"kind": "no-publisher", "value": ""},
                    "status": "cost-waived",
                })
        write_json(targets / "platform-signing-evidence.json", {
            "evidence": platform_entries,
            "schema_version": 1,
        })

        repository_url = f"https://github.com/{args.repository}"
        provenance = {
            "_type": "https://in-toto.io/Statement/v1",
            "predicate": {
                "buildDefinition": {
                    "buildType": f"{repository_url}/release/v1",
                    "externalParameters": {
                        "channel": "stable",
                        "packageVersion": args.package_version,
                        "productVersion": args.product_version,
                    },
                    "internalParameters": {"locked": True},
                    "resolvedDependencies": [{
                        "digest": {"gitCommit": args.candidate_sha},
                        "uri": f"git+{repository_url}@{args.candidate_sha}",
                    }],
                },
                "runDetails": {
                    "builder": {"id": f"{repository_url}/.github/workflows/release.yml@refs/heads/main"},
                    "metadata": {
                        "finishedOn": args.finished_on,
                        "invocationId": f"{repository_url}/actions/runs/{args.candidate_run_id}",
                        "startedOn": args.started_on,
                    },
                },
            },
            "predicateType": "https://slsa.dev/provenance/v1",
            "subject": [
                {"digest": {"sha256": archive_digests[name]}, "name": name}
                for name in names
            ],
        }
        write_json(targets / "provenance.intoto.json", provenance)

        manifest = {
            "classification": "feature",
            "license": "Apache-2.0",
            "migration_table_digest": f"sha256:{digest(targets / 'migration-table.json')}",
            "minimum_supported_harness_version": "0.8.0",
            "minimum_updater_version": "0.9.0",
            "platform_signing_evidence_digest": f"sha256:{digest(targets / 'platform-signing-evidence.json')}",
            "product": "aigent-hive",
            "provenance_digest": f"sha256:{digest(targets / 'provenance.intoto.json')}",
            "release_sequence": 9,
            "release_version": args.product_version,
            "schema_version": 1,
            "source": {
                "commit": args.candidate_sha,
                "repository": repository_url,
                "tag": f"v{args.product_version}",
            },
            "surface_inventory_digest": f"sha256:{digest(targets / 'release-surface-inventory.json')}",
        }
        write_json(targets / "bundle-manifest.json", manifest)

        requested_targets = []
        for path in sorted(targets.iterdir(), key=lambda item: item.name):
            requested_targets.append({
                "length": path.stat().st_size,
                "path": f"targets/{path.name}",
                "sha256": digest(path),
            })
        request = {
            "candidate": {
                "ref": args.candidate_ref,
                "run_id": args.candidate_run_id,
                "sha": args.candidate_sha,
            },
            "package_version": args.package_version,
            "product_version": args.product_version,
            "repository": repository_url,
            "schema_version": 1,
            "targets": requested_targets,
            "tuf_policy": {
                "consistent_snapshot": True,
                "root_threshold": 2,
                "root_total_authorities": 3,
                "roles": ["root", "snapshot", "targets", "timestamp"],
                "spec_version": "1.0.31",
            },
        }
        write_json(temporary / "signing-request.json", request)
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
    parser.add_argument("--started-on", required=True)
    parser.add_argument("--finished-on", required=True)
    parser.add_argument("--dist", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


if __name__ == "__main__":
    try:
        prepare(parse_args())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(f"release authorization preparation failed: {error}") from error
