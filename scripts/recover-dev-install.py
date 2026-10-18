#!/usr/bin/env python3
"""Nonshipping recovery of an authenticated legacy Codex developer installation.

Stage only verified generated package files in a reversible backup. The installed
CLI remains responsible for host activation, package updates and data migration.
No uninstall, manifest rewriting, directory deletion, host termination or settings
reset. Use plan, review its digest, stage, then the documented official CLI steps.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess

MANIFEST = ".hive/install/codex.json"
PACKAGE = ".hive/marketplaces/codex/plugins/aigent-hive/"
METADATA = ".hive/marketplaces/codex/.agents/plugins/marketplace.json"
KEEP = {"shared-marker", "canonical-data-protected"}


def digest(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def regular(path: Path) -> bytes:
    # Reject symlinks and Windows reparse points on every existing component.
    for part in [*reversed(path.parents), path]:
        info = part.lstat()
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError(f"linked path refused: {part}")
    if not path.is_file() or path.stat().st_size > 128 * 1024 * 1024:
        raise ValueError(f"not a bounded regular file: {path}")
    return path.read_bytes()


def relative_path(value: str) -> str:
    path = PurePosixPath(value)
    if (not value or "\\" in value or ":" in value or path.is_absolute()
            or any(p in ("", ".", "..") for p in value.split("/"))):
        raise ValueError("unsafe relative path")
    return value


def selected_paths(manifest: dict) -> list[dict]:
    if manifest.get("schema_version") != 1 or manifest.get("host") != "codex":
        raise ValueError("unsupported developer ownership manifest")
    result, seen = [], set()
    for entry in manifest["entries"]:
        path = relative_path(entry["path"])
        if path.casefold() in seen:
            raise ValueError("duplicate ownership path")
        seen.add(path.casefold())
        ownership = entry["ownership"]
        if ownership in KEEP:
            continue
        if not ((ownership == "immutable-plugin-package" and path.startswith(PACKAGE))
                or (ownership == "host-adapter-metadata" and path == METADATA)):
            raise ValueError(f"unsupported generated-file ownership: {path}")
        if not re.fullmatch(r"sha256:[0-9a-f]{64}", entry["digest"]):
            raise ValueError("invalid ownership digest")
        result.append({"path": path, "digest": entry["digest"]})
    if not result:
        raise ValueError("empty generated inventory")
    return result


def authenticate(root: Path, binary: Path) -> None:
    activation = root / ".hive/dev-install"
    target = regular(activation / "target-path").decode().strip()
    expected = regular(activation / "developer-sha256").decode().strip()
    if re.fullmatch(r"\\[0-9a-f]{64}", expected):
        expected = expected[1:]
    if not re.fullmatch(r"[0-9a-f]{64}", expected):
        raise ValueError("invalid developer activation digest")
    if Path(target).absolute() != binary or digest(regular(binary)) != "sha256:" + expected:
        raise ValueError("developer activation does not bind the supplied executable")
    # The actual installed CLI authenticates ownership and live bytes, not JSON alone.
    command = [str(binary), "install", "--scope", "user", "--hosts", "codex",
               "--validate", "--user-root", str(root), "--output", "json"]
    result = subprocess.run(command, capture_output=True, timeout=60, check=False)
    report = json.loads(result.stdout)
    if (result.returncode != 0 or report.get("code") != "hive.user-install-valid"
            or report.get("status") != "success" or report.get("changed_paths") != []):
        raise ValueError("installed CLI did not authenticate the current Codex projection")


def make_plan(root: Path, binary: Path) -> dict:
    before = regular(root / MANIFEST)
    authenticate(root, binary)
    if regular(root / MANIFEST) != before:
        raise ValueError("ownership changed during authentication")
    manifest = json.loads(before)
    entries = selected_paths(manifest)
    entries.append({"path": MANIFEST, "digest": digest(before)})
    for entry in entries:
        if digest(regular(root / entry["path"])) != entry["digest"]:
            raise ValueError("live generated file differs from authenticated ownership")
    return {"schema_version": 1, "user_root": str(root), "binary": str(binary),
            "binary_digest": digest(regular(binary)), "entries": entries}


def encode(value: dict) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode()


def write_new(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    # Validate the newly created destination tree without following junctions.
    for part in [*reversed(path.parent.parents), path.parent]:
        info = part.lstat()
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError("linked backup parent refused")
    with path.open("xb") as stream:
        stream.write(data)


def stage(plan: dict, backup: Path) -> None:
    root = Path(plan["user_root"])
    validate_plan(plan)
    for reserved in (root / ".hive", root / ".codex", root / ".agents"):
        if backup == reserved or reserved in backup.parents or backup in reserved.parents:
            raise ValueError("backup overlaps an installed namespace")
    if backup.exists():
        raise ValueError("backup already exists; preserve it and inspect its journal")
    # Revalidate all files before moving any of them. Copying alone is not a plan.
    for entry in plan["entries"]:
        relative_path(entry["path"])
        if digest(regular(root / entry["path"])) != entry["digest"]:
            raise ValueError("source changed after planning")
    write_new(backup / "plan.json", encode(plan))
    try:
        for entry in plan["entries"]:
            source = root / entry["path"]
            data = regular(source)
            if digest(data) != entry["digest"]:
                raise ValueError("source changed during staging")
            destination = backup / "files" / entry["path"]
            # A complete backup precedes removal of the live generated file.
            write_new(destination, data)
            if regular(source) != data:
                raise ValueError("source changed before staging")
            source.unlink()
        write_new(backup / "staged.json", encode({"staged": len(plan["entries"])}))
    except BaseException:
        restore(backup)
        raise


def restore(backup: Path) -> None:
    plan = json.loads(regular(backup / "plan.json"))
    validate_plan(plan)
    root = Path(plan["user_root"])
    # Preflight the entire recovery; never overwrite a subsequently changed file.
    available = []
    for entry in plan["entries"]:
        relative_path(entry["path"])
        saved = backup / "files" / entry["path"]
        if not saved.exists():
            continue
        data = regular(saved)
        if digest(data) != entry["digest"]:
            raise ValueError("damaged recovery backup")
        target = root / entry["path"]
        if target.exists() and regular(target) != data:
            raise ValueError("recovery target changed; no files overwritten")
        available.append((target, data))
    for target, data in available:
        if not target.exists():
            write_new(target, data)


def validate_plan(plan: dict) -> None:
    if plan.get("schema_version") != 1 or not Path(plan["user_root"]).is_absolute():
        raise ValueError("invalid recovery plan")
    seen = set()
    for entry in plan["entries"]:
        path = relative_path(entry["path"])
        if not (path.startswith(PACKAGE) or path in (METADATA, MANIFEST)):
            raise ValueError("recovery plan includes non-generated data")
        if path.casefold() in seen or not re.fullmatch(r"sha256:[0-9a-f]{64}", entry["digest"]):
            raise ValueError("invalid recovery entry")
        seen.add(path.casefold())
    if MANIFEST not in seen:
        raise ValueError("recovery plan lacks its ownership manifest")


def main() -> None:
    if os.name != "nt":
        raise ValueError("legacy recovery is qualified only for local Windows Codex")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["plan", "stage", "restore"])
    parser.add_argument("--user-root", type=Path)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--backup", type=Path, required=True)
    parser.add_argument("--confirm")
    args = parser.parse_args()
    backup = args.backup.absolute()
    if args.action == "restore":
        restore(backup)
        print(json.dumps({"restored": True, "backup_preserved": True}))
        return
    if args.user_root is None or args.binary is None:
        parser.error("--user-root and --binary are required")
    plan = make_plan(args.user_root.absolute(), args.binary.absolute())
    token = digest(encode(plan))
    if args.action == "stage":
        if args.confirm != token:
            raise ValueError("the exact fresh plan digest must be confirmed")
        stage(plan, backup)
    print(json.dumps({"action": args.action, "plan_digest": token,
                      "generated_file_count": len(plan["entries"]),
                      "changed_paths": [e["path"] for e in plan["entries"]],
                      "backup": str(backup)}, ensure_ascii=False))


if __name__ == "__main__":
    main()
