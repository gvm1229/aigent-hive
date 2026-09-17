#!/usr/bin/env python3
"""Source-only branch policy, Git hooks, and generated GitHub rule definition."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys


DEFAULTS = ("main", "develop")
CLASSES = ("feature", "fix", "release", "docs", "test", "refactor", "build", "chore")
PATTERN = rf"^({'|'.join(DEFAULTS)}|({'|'.join(CLASSES)})/.+)$"
ROOT = Path(__file__).resolve().parents[1]
RULESET_PATH = ROOT / ".github/branch-policy-ruleset.json"


class PolicyError(ValueError):
    """A rejected branch or an unsafe installation request."""


def git(repo: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True, text=True,
        encoding="utf-8", errors="replace", timeout=30,
    )
    if check and result.returncode:
        raise PolicyError(result.stderr.strip() or f"git {args[0]} failed")
    return result


def validate(name: str, repo: Path) -> None:
    # Inspect the literal input: --branch would expand @{-1} before validation.
    if not re.fullmatch(PATTERN, name):
        raise PolicyError(
            f"branch {name!r} is prohibited; use main, develop, or "
            + ", ".join(f"{kind}/<name>" for kind in CLASSES)
        )
    if git(repo, "check-ref-format", f"refs/heads/{name}", check=False).returncode:
        raise PolicyError(f"invalid Git branch syntax: {name!r}")


def current(repo: Path) -> str:
    result = git(repo, "symbolic-ref", "--quiet", "--short", "HEAD", check=False)
    if result.returncode:
        raise PolicyError("detached HEAD: provide an explicit branch name for CI validation")
    name = result.stdout.strip()
    validate(name, repo)
    return name


def validate_pr(head: str, base: str, head_repo: str, base_repo: str, repo: Path) -> None:
    if not head_repo or not base_repo:
        raise PolicyError("PR repository identity is required")
    validate(head, repo)
    validate(base, repo)
    if base == "main" and (head != "develop" or head_repo != base_repo):
        raise PolicyError("main accepts only develop from the same repository")


def zero_oid(value: str) -> bool:
    return len(value) in (40, 64) and set(value) == {"0"}


def validate_transaction(lines: str, state: str, repo: Path) -> None:
    if state != "prepared":
        return
    for line in lines.splitlines():
        parts = line.split()
        if len(parts) != 3:
            raise PolicyError("malformed reference-transaction input")
        _old, new, ref = parts
        if ref.startswith("refs/heads/") and not zero_oid(new):
            validate(ref[len("refs/heads/"):], repo)


def validate_push(lines: str, repo: Path) -> None:
    for line in lines.splitlines():
        parts = line.split()
        if len(parts) != 4:
            raise PolicyError("malformed pre-push input")
        local, local_oid, remote, _remote_oid = parts
        if local.startswith("refs/heads/") and not zero_oid(local_oid):
            validate(local[len("refs/heads/"):], repo)
        if remote.startswith("refs/heads/") and not zero_oid(local_oid):
            validate(remote[len("refs/heads/"):], repo)


def ruleset() -> dict:
    # This repository rejects branch_name_pattern (HTTP 422). Restrict every
    # non-allowlisted ref instead; Git itself already validates ref syntax.
    allowed = [f"refs/heads/{name}" for name in DEFAULTS]
    allowed += [f"refs/heads/{kind}/{suffix}" for kind in CLASSES for suffix in ("*", "**/*")]
    return {
        "name": "Hive branch naming", "target": "branch", "enforcement": "active",
        "bypass_actors": [],
        "conditions": {"ref_name": {"include": ["~ALL"], "exclude": allowed}},
        "rules": [{"type": "creation"}, {"type": "update"}],
    }


def ruleset_text() -> str:
    return json.dumps(ruleset(), indent=2) + "\n"


def hook_bytes(event: str) -> bytes:
    python = shlex.quote(Path(sys.executable).resolve().as_posix())
    script = shlex.quote(Path(__file__).resolve().as_posix())
    return (
        "#!/bin/sh\n# Hive source branch policy v1\n"
        f'exec {python} {script} {event} "$@"\n'
    ).encode("utf-8")


def install_hooks(repo: Path, apply: bool) -> list[str]:
    configured = git(repo, "config", "--get", "core.hooksPath", check=False)
    if configured.returncode == 0:
        raise PolicyError("core.hooksPath already configured; preserve it and integrate manually")
    if configured.returncode != 1:
        raise PolicyError("cannot inspect core.hooksPath")
    directory = Path(git(repo, "rev-parse", "--path-format=absolute", "--git-path", "hooks").stdout.strip())
    for ancestor in (directory, *directory.parents):
        if ancestor.is_symlink():
            raise PolicyError("refusing a symlink in the hooks directory path")
    planned = []
    for event in ("reference-transaction", "pre-push", "pre-commit"):
        path = directory / event
        expected = hook_bytes("current" if event == "pre-commit" else event)
        if path.is_symlink() or (path.exists() and not path.is_file()):
            raise PolicyError(f"unsafe existing hook: {path}")
        if path.exists():
            if path.read_bytes() != expected:
                raise PolicyError(f"existing hook preserved: {path}")
            if os.name != "nt" and not path.stat().st_mode & 0o111:
                raise PolicyError(f"existing hook is not executable: {path}")
        else:
            planned.append((path, expected))
    if apply:
        directory.mkdir(parents=True, exist_ok=True)
        written = []
        try:
            for path, expected in planned:
                with path.open("xb") as stream:
                    written.append((path, expected))
                    stream.write(expected)
                path.chmod(0o755)
        except OSError:
            # Only remove the exact hook bytes created by this invocation.
            for path, expected in reversed(written):
                if not path.is_symlink() and path.is_file() and path.read_bytes() == expected:
                    path.unlink()
            raise
    return [str(path) for path, _ in planned]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("check")
    check.add_argument("names", nargs="+")
    commands.add_parser("current")
    create = commands.add_parser("create")
    create.add_argument("name")
    rename = commands.add_parser("rename")
    rename.add_argument("old")
    rename.add_argument("new")
    transaction = commands.add_parser("reference-transaction")
    transaction.add_argument("state", choices=("prepared", "committed", "aborted"))
    push = commands.add_parser("pre-push")
    push.add_argument("remote", nargs="?")
    push.add_argument("url", nargs="?")
    pr = commands.add_parser("pr")
    for field in ("head", "base", "head-repo", "base-repo"):
        pr.add_argument("--" + field, required=True)
    install = commands.add_parser("install-hooks")
    install.add_argument("--apply", action="store_true")
    generate = commands.add_parser("ruleset")
    generate.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.command == "check":
            for name in args.names:
                validate(name, args.repo)
        elif args.command == "current":
            print(current(args.repo))
        elif args.command == "create":
            validate(args.name, args.repo)
            if args.name in DEFAULTS:
                raise PolicyError("use Git directly for explicitly authorized bootstrap")
            git(args.repo, "switch", "-c", args.name)
        elif args.command == "rename":
            validate(args.new, args.repo)
            if args.old in DEFAULTS or args.new in DEFAULTS:
                raise PolicyError("renaming main/develop is outside the work-branch helper")
            git(args.repo, "branch", "-m", "--", args.old, args.new)
        elif args.command == "reference-transaction":
            validate_transaction(sys.stdin.read(), args.state, args.repo)
        elif args.command == "pre-push":
            validate_push(sys.stdin.read(), args.repo)
        elif args.command == "pr":
            validate_pr(args.head, args.base, args.head_repo, args.base_repo, args.repo)
        elif args.command == "install-hooks":
            print(json.dumps({"apply": args.apply, "paths": install_hooks(args.repo, args.apply)}))
        elif args.command == "ruleset":
            expected = ruleset_text()
            if args.check:
                if RULESET_PATH.read_text(encoding="utf-8") != expected:
                    raise PolicyError("generated branch ruleset is stale")
            else:
                print(expected, end="")
    except (PolicyError, OSError, subprocess.SubprocessError) as error:
        print(f"branch-policy: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
