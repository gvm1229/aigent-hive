#!/usr/bin/env python3
"""Check repository Markdown links and GitHub-style heading anchors."""

from __future__ import annotations

import argparse
import hashlib
import html
import json
import re
import subprocess
import sys
import unicodedata
from dataclasses import asdict, dataclass
from pathlib import Path
from urllib.parse import unquote


MARKDOWN_SUFFIXES = (".md", ".md.jinja")
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
REFERENCE_LINK = re.compile(r"^\s*\[[^\]]+\]:\s*(\S+)", re.MULTILINE)
HTML_LINK = re.compile(r"""(?i)\b(?:href|src)\s*=\s*["']([^"']+)["']""")
HEADING = re.compile(r"^[ \t]{0,3}#{1,6}[ \t]+(.+?)[ \t]*#*[ \t]*$")
EXPLICIT_ID = re.compile(r"""(?i)\b(?:id|name)\s*=\s*["']([^"']+)["']""")
FENCE = re.compile(r"^[ \t]{0,3}(`{3,}|~{3,})")
EXTERNAL = re.compile(r"^[a-z][a-z0-9+.-]*:", re.IGNORECASE)
LINE_ANCHOR = re.compile(r"(?i)^L\d+(?:-L\d+)?$")
ARCHIVE_NAVIGATION = {
    "docs/archive/README.md",
    "docs/archive/MANIFEST.md",
}
# Preserve this exact upstream typo without changing the pinned third-party snapshot.
# Any source byte, path, or destination change restores ordinary link validation.
PRESERVED_UPSTREAM_LINKS = {
    (
        "vendor/process-wrap-9.1.0/CHANGELOG.md",
        "04fea1ee8b6498c0aee58f75ff2d5036bb4a6549f6122ea75199ed3f6172104c",
        "doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html#tymethod.process_group",
    ): "upstream URL omits https; pinned source bytes preserved",
}


@dataclass(frozen=True)
class Failure:
    source: str
    line: int
    target: str
    reason: str


def inventory(root: Path) -> list[Path]:
    completed = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root,
        check=False,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.decode("utf-8", errors="replace").strip())
    candidates = (
        root / entry.decode("utf-8")
        for entry in completed.stdout.split(b"\0")
        if entry and entry.decode("utf-8").endswith(MARKDOWN_SUFFIXES)
    )
    return sorted(
        path
        for path in candidates
        if path.is_file()
        and (
            not path.relative_to(root).as_posix().startswith("docs/archive/")
            or path.relative_to(root).as_posix() in ARCHIVE_NAVIGATION
        )
    )


def visible_lines(text: str) -> list[tuple[int, str]]:
    visible: list[tuple[int, str]] = []
    fence_character: str | None = None
    fence_length = 0
    in_comment = False
    for line_number, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line
        if in_comment:
            if "-->" in line:
                line = line.split("-->", 1)[1]
                in_comment = False
            else:
                continue
        while "<!--" in line:
            before, after = line.split("<!--", 1)
            if "-->" in after:
                line = before + after.split("-->", 1)[1]
            else:
                line = before
                in_comment = True
                break
        match = FENCE.match(line)
        if fence_character is not None:
            if (
                match
                and match.group(1)[0] == fence_character
                and len(match.group(1)) >= fence_length
            ):
                fence_character = None
                fence_length = 0
            continue
        if match:
            fence_character = match.group(1)[0]
            fence_length = len(match.group(1))
            continue
        visible.append((line_number, line))
    return visible


def slug_base(heading: str) -> str:
    value = html.unescape(re.sub(r"<[^>]+>", "", heading)).casefold()
    value = re.sub(r"`+([^`]*)`+", r"\1", value)
    value = re.sub(r"!\[([^\]]*)\]\([^)]*\)", r"\1", value)
    value = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", value)
    value = "".join(
        character
        for character in value
        if character in "-_ "
        or character.isspace()
        or not unicodedata.category(character).startswith(("P", "S"))
    )
    return re.sub(r"\s", "-", value.strip())


def anchors(text: str) -> set[str]:
    result: set[str] = set()
    duplicate_counts: dict[str, int] = {}
    for _, line in visible_lines(text):
        for explicit in EXPLICIT_ID.findall(line):
            result.add(html.unescape(explicit).casefold())
        match = HEADING.match(line)
        if match is None:
            continue
        base = slug_base(match.group(1))
        count = duplicate_counts.get(base, 0)
        duplicate_counts[base] = count + 1
        result.add(base if count == 0 else f"{base}-{count}")
    return result


def destinations(text: str) -> list[tuple[int, str]]:
    found: list[tuple[int, str]] = []
    for line_number, line in visible_lines(text):
        without_code = re.sub(r"`+[^`]*`+", "", line)
        targets = INLINE_LINK.findall(without_code) + HTML_LINK.findall(without_code)
        for raw_target in targets:
            target = raw_target.strip()
            if target.startswith("<") and ">" in target:
                target = target[1 : target.index(">")]
            else:
                target = target.split(maxsplit=1)[0]
            found.append((line_number, target))
    visible_text = "\n".join(line for _, line in visible_lines(text))
    for match in REFERENCE_LINK.finditer(visible_text):
        line_number = visible_text.count("\n", 0, match.start()) + 1
        found.append((line_number, match.group(1).strip("<>")))
    return found


def resolve_path(root: Path, source: Path, target: str) -> Path:
    relative = unquote(target)
    if relative.startswith("/"):
        return root / relative.lstrip("/")
    return source.parent / relative


def scan(root: Path) -> dict[str, object]:
    root = root.resolve()
    documents = inventory(root)
    anchor_cache: dict[Path, set[str]] = {}
    failures: list[Failure] = []
    preserved_upstream_links: list[dict[str, object]] = []
    checked_links = 0
    for source in documents:
        text = source.read_text(encoding="utf-8")
        source_digest = hashlib.sha256(source.read_bytes()).hexdigest()
        for line_number, target in destinations(text):
            preserved = PRESERVED_UPSTREAM_LINKS.get((
                source.relative_to(root).as_posix(), source_digest, target,
            ))
            if preserved:
                preserved_upstream_links.append({
                    "source": source.relative_to(root).as_posix(), "line": line_number,
                    "target": target, "source_sha256": source_digest, "reason": preserved,
                })
                continue
            if (
                not target
                or target.startswith("//")
                or EXTERNAL.match(target)
            ):
                continue
            checked_links += 1
            path_text, separator, anchor = target.partition("#")
            destination = resolve_path(root, source, path_text) if path_text else source
            if not destination.exists():
                failures.append(
                    Failure(
                        source.relative_to(root).as_posix(),
                        line_number,
                        target,
                        "missing-target",
                    )
                )
                continue
            if (
                separator
                and anchor
                and destination.is_file()
                and destination.name.endswith(MARKDOWN_SUFFIXES)
                and not LINE_ANCHOR.fullmatch(anchor)
            ):
                available = anchor_cache.setdefault(
                    destination.resolve(),
                    anchors(destination.read_text(encoding="utf-8")),
                )
                normalized_anchor = unquote(anchor).casefold()
                if normalized_anchor not in available:
                    failures.append(
                        Failure(
                            source.relative_to(root).as_posix(),
                            line_number,
                            target,
                            "missing-anchor",
                        )
                    )
    return {
        "schema_version": 1,
        "checked_files": len(documents),
        "checked_links": checked_links,
        "failure_count": len(failures),
        "failures": [asdict(failure) for failure in failures],
        "preserved_upstream_links": preserved_upstream_links,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--output", choices=("json",), default="json")
    arguments = parser.parse_args()
    try:
        report = scan(arguments.root)
    except (OSError, RuntimeError, UnicodeError) as error:
        print(json.dumps({"schema_version": 1, "error": str(error)}))
        return 2
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 1 if report["failure_count"] else 0


if __name__ == "__main__":
    sys.exit(main())
