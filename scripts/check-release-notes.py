#!/usr/bin/env python3
"""Validate the canonical bilingual GitHub Release description."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


ENGLISH_HEADINGS = ("Scope", "Compatibility", "Verification", "Publication")
KOREAN_HEADINGS = ("범위", "호환성", "검증", "게시")
ENGLISH_BANNED_WORDS = re.compile(
    r"\b(?:we|you|please|can|may|might|easy|easily|simply|very|etc)\b", re.IGNORECASE
)
KOREAN_COMMON_ENGLISH = re.compile(
    r"\b(?:the|and|with|for|from|when|use|used|release|scope|compatibility|verification|publication)\b",
    re.IGNORECASE,
)
BULLET = re.compile(r"^- \[([A-Z][A-Z0-9-]+)\] (.+)$")


@dataclass(frozen=True)
class ReleaseSection:
    language: str
    heading: str
    facts: dict[str, str]


def fail(message: str) -> None:
    raise ValueError(f"release note error: {message}")


def sections_after(lines: list[str], marker: str) -> list[str]:
    try:
        index = lines.index(marker)
    except ValueError as error:
        fail(f"missing required heading {marker}")
        raise AssertionError from error
    return lines[index + 1 :]


def parse_language(
    lines: list[str], language_heading: str, headings: tuple[str, ...], language: str
) -> list[ReleaseSection]:
    content = sections_after(lines, language_heading)
    if language_heading == "## English":
        try:
            content = content[: content.index("## 한국어")]
        except ValueError as error:
            fail("missing required heading ## 한국어")
            raise AssertionError from error
    sections: list[ReleaseSection] = []
    for position, heading in enumerate(headings):
        marker = f"### {heading}"
        try:
            start = content.index(marker)
        except ValueError as error:
            fail(f"missing {language} section {heading}")
            raise AssertionError from error
        end = len(content)
        for following in headings[position + 1 :]:
            following_marker = f"### {following}"
            if following_marker in content[start + 1 :]:
                end = content.index(following_marker, start + 1)
                break
        body = [line for line in content[start + 1 : end] if line.strip()]
        facts: dict[str, str] = {}
        for line in body:
            match = BULLET.fullmatch(line)
            if match is None:
                fail(f"{language} {heading} must contain only fact-ID bullets")
            fact_id, fact = match.groups()
            if fact_id in facts:
                fail(f"{language} {heading} repeats fact ID {fact_id}")
            facts[fact_id] = fact
        if not facts:
            fail(f"{language} {heading} has no facts")
        sections.append(ReleaseSection(language, heading, facts))
    return sections


def validate_order(lines: list[str]) -> None:
    required = ["## English", *[f"### {heading}" for heading in ENGLISH_HEADINGS], "## 한국어", *[f"### {heading}" for heading in KOREAN_HEADINGS]]
    positions: list[int] = []
    for heading in required:
        try:
            positions.append(lines.index(heading))
        except ValueError as error:
            fail(f"missing required heading {heading}")
            raise AssertionError from error
    if positions != sorted(positions):
        fail("sections must be English first and Korean second")


def validate_english(section: ReleaseSection) -> None:
    for fact_id, fact in section.facts.items():
        if any(ord(character) > 127 for character in fact):
            fail(f"English {section.heading} fact {fact_id} contains non-ASCII text")
        if len(fact.split()) > 25:
            fail(f"English {section.heading} fact {fact_id} exceeds 25 words")
        if ENGLISH_BANNED_WORDS.search(fact):
            fail(f"English {section.heading} fact {fact_id} is not ASD-STE100 concise")


def validate_korean(section: ReleaseSection) -> None:
    for fact_id, fact in section.facts.items():
        plain_fact = re.sub(r"`[^`]+`", "", fact)
        if not re.search(r"[가-힣]", plain_fact):
            fail(f"Korean {section.heading} fact {fact_id} has no Korean explanation")
        if KOREAN_COMMON_ENGLISH.search(plain_fact):
            fail(f"Korean {section.heading} fact {fact_id} contains ordinary English prose")


def check(path: Path, version: str) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != f"# Aigent Hive {version}":
        fail(f"title must be # Aigent Hive {version}")
    validate_order(lines)
    english = parse_language(lines, "## English", ENGLISH_HEADINGS, "English")
    korean = parse_language(lines, "## 한국어", KOREAN_HEADINGS, "Korean")
    for english_section, korean_section in zip(english, korean, strict=True):
        validate_english(english_section)
        validate_korean(korean_section)
        if set(english_section.facts) != set(korean_section.facts):
            fail(
                f"fact IDs differ for {english_section.heading}/{korean_section.heading}: "
                f"English={sorted(english_section.facts)}, Korean={sorted(korean_section.facts)}"
            )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, help="Exact X.Y.Z product version")
    parser.add_argument("--path", required=True, type=Path, help="Release note Markdown path")
    arguments = parser.parse_args()
    try:
        check(arguments.path, arguments.version)
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
