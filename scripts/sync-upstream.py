#!/usr/bin/env python3
"""Vendor pinned upstream Skill sources byte-for-byte and report what changed.

`harness/upstream/UPSTREAM.lock` lists each source: repository, path, branch, pinned commit,
license and the Hive Skills that use it. The default mode fetches each source's latest commit,
replaces `harness/upstream/<name>/` with the exact upstream files (plus the license and notice
files named in the lock) and updates the lock. `--check` fetches and compares without writing.
One unreachable or vanished source is skipped with a reason; the other sources still sync.

Hive-owned overrides, conflict rulings and user preferences live outside `harness/upstream/`, so a
sync never touches them. `scripts/sync-user-plugin.py` copies the vendored trees into the Skills
that use them.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
LOCK_NAME = "UPSTREAM.lock"
REQUIRED = ("name", "repo", "ref", "path", "commit", "license", "license_files", "used_by", "synced_at")
NAME = re.compile(r"[a-z0-9][a-z0-9-]*")
REPO = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+")
CONFLICTS = "harness/skills/design-curate/references/conflicts.md"


class SyncError(ValueError):
    """Invalid lock or vendored tree."""


def safe_relative(value: str, label: str) -> PurePosixPath:
    path = PurePosixPath(value)
    if not value or path.is_absolute() or "\\" in value or any(part in ("", ".", "..") for part in path.parts):
        raise SyncError(f"unsafe {label}: {value!r}")
    return path


def load_lock(root: Path) -> dict:
    path = root / "harness/upstream" / LOCK_NAME
    try:
        lock = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SyncError(f"cannot read {path}: {error}") from error
    if lock.get("schema_version") != 1 or not isinstance(lock.get("sources"), list):
        raise SyncError("upstream lock must have schema_version 1 and a sources list")
    names = set()
    for source in lock["sources"]:
        missing = [key for key in REQUIRED if key not in source]
        if missing:
            raise SyncError(f"upstream source lacks {', '.join(missing)}")
        if not NAME.fullmatch(source["name"]) or source["name"] in names:
            raise SyncError(f"invalid or duplicate upstream name: {source['name']}")
        names.add(source["name"])
        if not REPO.fullmatch(source["repo"]):
            raise SyncError(f"invalid upstream repository: {source['repo']}")
        safe_relative(source["path"], "upstream path")
        if not source["license_files"]:
            raise SyncError(f"{source['name']} must vendor at least one license file")
        for item in source["license_files"]:
            safe_relative(item, "license file")
        if not source["used_by"] or not all(NAME.fullmatch(skill) for skill in source["used_by"]):
            raise SyncError(f"{source['name']} has no valid used_by Skill")
    return lock


def write_lock(root: Path, lock: dict) -> None:
    path = root / "harness/upstream" / LOCK_NAME
    path.write_text(json.dumps(lock, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")


def repository_url(source: dict) -> str:
    base = os.environ.get("HIVE_UPSTREAM_BASE_URL", "https://github.com").rstrip("/")
    return f"{base}/{source['repo']}.git"


def git(*args: str, cwd: Path | None = None) -> str:
    result = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, timeout=600,
                            env={**os.environ, "GIT_TERMINAL_PROMPT": "0"})
    if result.returncode:
        raise RuntimeError((result.stderr or result.stdout).strip().splitlines()[-1:] or ["git failed"])
    return result.stdout.strip()


def expected_files(checkout: Path, source: dict) -> dict[str, bytes]:
    """Map vendored relative paths to exact upstream bytes."""
    base = checkout.joinpath(*safe_relative(source["path"], "upstream path").parts)
    if not base.is_dir():
        raise FileNotFoundError(f"path {source['path']} is missing upstream")
    files: dict[str, bytes] = {}
    for path in sorted(base.rglob("*")):
        if path.is_symlink():
            raise SyncError(f"{source['name']} contains a symbolic link: {path.relative_to(base)}")
        if path.is_file():
            files[path.relative_to(base).as_posix()] = path.read_bytes()
    for item in source["license_files"]:
        relative = safe_relative(item, "license file")
        upstream = checkout.joinpath(*relative.parts)
        if not upstream.is_file():
            raise FileNotFoundError(f"license file {item} is missing upstream")
        files.setdefault(relative.name, upstream.read_bytes())
    return files


def tree_files(directory: Path) -> dict[str, bytes]:
    if not directory.is_dir():
        return {}
    return {path.relative_to(directory).as_posix(): path.read_bytes()
            for path in sorted(directory.rglob("*")) if path.is_file()}


def replace_tree(directory: Path, files: dict[str, bytes]) -> None:
    if directory.exists():
        shutil.rmtree(directory)
    for relative, payload in files.items():
        target = directory.joinpath(*PurePosixPath(relative).parts)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(payload)


def diff_summary(old: dict[str, bytes], new: dict[str, bytes]) -> dict[str, list[str]]:
    return {
        "added": sorted(set(new) - set(old)),
        "removed": sorted(set(old) - set(new)),
        "modified": sorted(name for name in set(old) & set(new) if old[name] != new[name]),
    }


def conflict_topics(root: Path, name: str) -> list[str]:
    """Return conflict-ruling topics that cite one upstream source by name."""
    path = root / CONFLICTS
    if not path.is_file():
        return []
    topics = []
    for line in path.read_text(encoding="utf-8").splitlines():
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) < 4 or cells[0] in ("Topic", "주제") or set(cells[0]) <= {"-", " "}:
            continue
        if re.search(r"(?<![a-z0-9-])" + re.escape(name) + r"(?![a-z0-9-])", cells[2]):
            topics.append(cells[0])
    return topics


def sync(root: Path, *, check: bool, only: set[str] | None = None, today: str | None = None) -> dict:
    lock = load_lock(root)
    results = []
    changed_any = False
    for source in lock["sources"]:
        if only and source["name"] not in only:
            continue
        entry = {"name": source["name"], "repo": source["repo"], "used_by": source["used_by"],
                 "previous_commit": source["commit"]}
        with tempfile.TemporaryDirectory(prefix="hive-upstream-") as temporary:
            checkout = Path(temporary) / "checkout"
            try:
                git("clone", "--quiet", "--depth", "1", "--branch", source["ref"], repository_url(source), str(checkout))
                commit = git("rev-parse", "HEAD", cwd=checkout)
                files = expected_files(checkout, source)
            except (RuntimeError, FileNotFoundError, subprocess.TimeoutExpired) as error:
                entry.update(status="skipped", reason=str(error).strip("[]'\""))
                results.append(entry)
                continue
        destination = root / "harness/upstream" / source["name"]
        changes = diff_summary(tree_files(destination), files)
        changed = any(changes.values())
        entry.update(commit=commit, changes=changes,
                     status="changed" if changed else ("unchanged" if commit == source["commit"] else "commit-only"))
        if changed:
            changed_any = True
            entry["conflict_topics"] = conflict_topics(root, source["name"])
        if not check and (changed or commit != source["commit"]):
            replace_tree(destination, files)
            source["commit"] = commit
            source["synced_at"] = today or dt.date.today().isoformat()
        results.append(entry)
    if not check:
        write_lock(root, lock)
    return {"schema_version": 1, "mode": "check" if check else "sync", "changed": changed_any, "sources": results}


def verify(root: Path) -> list[str]:
    """Offline check: every vendored tree exists and carries its license files."""
    failures = []
    for source in load_lock(root)["sources"]:
        directory = root / "harness/upstream" / source["name"]
        if not directory.is_dir():
            failures.append(f"{source['name']}: vendored tree missing")
            continue
        for item in source["license_files"]:
            if not (directory / PurePosixPath(item).name).is_file():
                failures.append(f"{source['name']}: {PurePosixPath(item).name} missing")
    return failures


def pull_request_body(report: dict) -> str:
    lines = ["Weekly upstream sync. Vendored files are byte-for-byte copies of the pinned commits.", ""]
    for entry in report["sources"]:
        if entry["status"] == "skipped":
            lines.append(f"- `{entry['name']}` ({entry['repo']}): skipped, {entry['reason']}")
            continue
        if entry["status"] not in ("changed", "commit-only"):
            continue
        counts = ", ".join(f"{len(entry['changes'][kind])} {kind}" for kind in ("added", "modified", "removed"))
        lines.append(f"- `{entry['name']}` ({entry['repo']}): `{entry['previous_commit'][:12]}` → "
                     f"`{entry['commit'][:12]}`; {counts}; used by {', '.join(entry['used_by'])}")
        for topic in entry.get("conflict_topics", []):
            lines.append(f"  - Revisit conflict ruling: {topic}")
    lines += ["", "Before merging, reconcile `hive-overrides.md` and `references/conflicts.md` with the changes."]
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--check", action="store_true", help="fetch and compare without writing")
    parser.add_argument("--verify", action="store_true", help="offline check of vendored trees and licenses")
    parser.add_argument("--only", action="append", help="sync only this source name (repeatable)")
    parser.add_argument("--pr-body", type=Path, help="write a pull request body summarizing changes")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    try:
        if args.verify:
            failures = verify(root)
            print(json.dumps({"schema_version": 1, "status": "invalid" if failures else "valid",
                              "failures": failures}, indent=2))
            return 1 if failures else 0
        report = sync(root, check=args.check, only=set(args.only) if args.only else None)
    except SyncError as error:
        print(json.dumps({"schema_version": 1, "status": "invalid", "message": str(error)}))
        return 2
    if args.pr_body:
        args.pr_body.write_text(pull_request_body(report), encoding="utf-8")
    print(json.dumps(report, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
