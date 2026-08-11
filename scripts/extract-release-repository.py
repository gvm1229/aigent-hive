#!/usr/bin/env python3
"""Safely extract a bounded external TUF repository archive."""

from __future__ import annotations

import argparse
import os
from pathlib import Path, PurePosixPath
import shutil
import tarfile
import tempfile


MAX_MEMBERS = 512
MAX_FILE_BYTES = 512 * 1024 * 1024
MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024
ALLOWED_NAMESPACES = {"metadata", "targets"}


def member_path(name: str) -> PurePosixPath:
    if not name or "\\" in name or "\x00" in name:
        raise ValueError("repository member has a non-portable path")
    path = PurePosixPath(name)
    if path.is_absolute() or any(part in ("", ".", "..") for part in path.parts):
        raise ValueError("repository member escapes its archive namespace")
    if path.parts[0] not in ALLOWED_NAMESPACES:
        raise ValueError("repository member is outside metadata/ or targets/")
    return path


def extract(archive: Path, output: Path) -> None:
    if not archive.is_file():
        raise ValueError("repository archive is not a regular file")
    if output.exists():
        raise ValueError("repository output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix="tuf-repository-", dir=output.parent))
    try:
        with tarfile.open(archive, mode="r:gz") as source:
            members = source.getmembers()
            if not members or len(members) > MAX_MEMBERS:
                raise ValueError("repository archive has an invalid member count")
            seen: set[str] = set()
            seen_portable: set[str] = set()
            total = 0
            validated: list[tuple[tarfile.TarInfo, PurePosixPath]] = []
            for member in members:
                path = member_path(member.name)
                portable = path.as_posix().casefold()
                if path.as_posix() in seen or portable in seen_portable:
                    raise ValueError("repository archive contains a duplicate portable path")
                seen.add(path.as_posix())
                seen_portable.add(portable)
                if not (member.isdir() or member.isreg()):
                    raise ValueError("repository archive contains a link or special file")
                if member.size < 0 or member.size > MAX_FILE_BYTES:
                    raise ValueError("repository member exceeds the size limit")
                total += member.size
                if total > MAX_TOTAL_BYTES:
                    raise ValueError("repository archive exceeds the total size limit")
                validated.append((member, path))

            for member, path in validated:
                destination = temporary.joinpath(*path.parts)
                if member.isdir():
                    destination.mkdir(parents=True, exist_ok=False)
                    destination.chmod(0o755)
                    continue
                destination.parent.mkdir(parents=True, exist_ok=True)
                stream = source.extractfile(member)
                if stream is None:
                    raise ValueError("repository file payload is unavailable")
                with stream, destination.open("xb") as target:
                    shutil.copyfileobj(stream, target, length=1024 * 1024)
                if destination.stat().st_size != member.size:
                    raise ValueError("repository member length changed during extraction")
                destination.chmod(0o644)
        for required in (
            "metadata/root.json",
            "metadata/snapshot.json",
            "metadata/targets.json",
            "metadata/timestamp.json",
        ):
            if not temporary.joinpath(*PurePosixPath(required).parts).is_file():
                raise ValueError(f"repository archive omits {required}")
        os.replace(temporary, output)
    except Exception:
        shutil.rmtree(temporary, ignore_errors=True)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


if __name__ == "__main__":
    try:
        arguments = parse_args()
        extract(arguments.archive, arguments.output)
    except (OSError, ValueError, tarfile.TarError) as error:
        raise SystemExit(f"release repository extraction failed: {error}") from error
