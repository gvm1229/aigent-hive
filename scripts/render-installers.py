#!/usr/bin/env python3
"""Render version- and digest-pinned direct installers from npm artifacts."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re
import stat


ROOT = Path(__file__).resolve().parents[1]
EXACT_VERSION = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
TARGETS = {
    "aarch64-apple-darwin": (
        "aigent-hive-darwin-arm64-{version}.tgz",
        "__AIGENT_HIVE_SHA256_AARCH64_APPLE_DARWIN__",
    ),
    "x86_64-apple-darwin": (
        "aigent-hive-darwin-x64-{version}.tgz",
        "__AIGENT_HIVE_SHA256_X86_64_APPLE_DARWIN__",
    ),
    "aarch64-unknown-linux-musl": (
        "aigent-hive-linux-arm64-{version}.tgz",
        "__AIGENT_HIVE_SHA256_AARCH64_UNKNOWN_LINUX_MUSL__",
    ),
    "x86_64-unknown-linux-musl": (
        "aigent-hive-linux-x64-{version}.tgz",
        "__AIGENT_HIVE_SHA256_X86_64_UNKNOWN_LINUX_MUSL__",
    ),
    "x86_64-pc-windows-msvc": (
        "aigent-hive-win32-x64-{version}.tgz",
        "__AIGENT_HIVE_SHA256_X86_64_PC_WINDOWS_MSVC__",
    ),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--dist", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def read_digest(dist: Path, version: str, name_template: str) -> str:
    name = name_template.format(version=version)
    artifact = dist / name
    if not artifact.is_file() or artifact.is_symlink():
        raise SystemExit(f"missing regular npm artifact: {artifact}")
    with artifact.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def render(source: Path, destination: Path, replacements: dict[str, str]) -> None:
    if destination.exists() or destination.is_symlink():
        raise SystemExit(f"installer destination already exists: {destination}")
    text = source.read_text(encoding="utf-8")
    for marker, value in replacements.items():
        if text.count(marker) != 1:
            raise SystemExit(f"installer marker count differs from one: {source}:{marker}")
        text = text.replace(marker, value)
    unresolved = sorted(set(re.findall(r"__AIGENT_HIVE_[A-Z0-9_]+__", text)))
    if unresolved:
        raise SystemExit(f"installer contains unresolved markers: {source}:{unresolved}")
    destination.write_text(text, encoding="utf-8", newline="\n")


def main() -> None:
    args = parse_args()
    if not EXACT_VERSION.fullmatch(args.version):
        raise SystemExit("--version must be an exact X.Y.Z version")
    if not args.dist.is_dir() or args.dist.is_symlink():
        raise SystemExit("--dist must be a regular directory")
    args.output.mkdir(parents=True, exist_ok=True)
    if args.output.is_symlink():
        raise SystemExit("--output must not be a symlink")

    replacements = {"__AIGENT_HIVE_VERSION__": args.version}
    for _, (name_template, marker) in TARGETS.items():
        replacements[marker] = read_digest(args.dist, args.version, name_template)

    render(
        ROOT / "scripts/install.sh",
        args.output / "install.sh",
        {
            **{key: value for key, value in replacements.items() if "WINDOWS" not in key},
            "__AIGENT_HIVE_APPLE_TEAM_ID__": "",
        },
    )
    render(
        ROOT / "scripts/install.ps1",
        args.output / "install.ps1",
        {
            "__AIGENT_HIVE_VERSION__": args.version,
            "__AIGENT_HIVE_SHA256_X86_64_PC_WINDOWS_MSVC__": replacements[
                "__AIGENT_HIVE_SHA256_X86_64_PC_WINDOWS_MSVC__"
            ],
            "__AIGENT_HIVE_WINDOWS_CERTIFICATE_THUMBPRINT__": "",
        },
    )
    render(
        ROOT / "scripts/install.cmd",
        args.output / "install.cmd",
        {"__AIGENT_HIVE_VERSION__": args.version},
    )
    shell_mode = (args.output / "install.sh").stat().st_mode
    (args.output / "install.sh").chmod(shell_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


if __name__ == "__main__":
    main()
