#!/usr/bin/env python3
"""Inventory and check every documentation candidate in the Hive workspace."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


CANDIDATE_SUFFIXES = (".md", ".md.jinja", ".txt", ".rst", ".adoc")
CANDIDATE_PREFIXES = ("README", "LICENSE", "NOTICE", "CHANGELOG")
DISPOSITIONS = {
    "edit-project-source",
    "regenerate-from-source",
    "no-change-english",
    "no-change-compliant",
    "no-change-protected-exact",
    "no-change-generated",
    "fixture-semantic-preserve",
}
ALLOWLIST_PATH = Path(__file__).with_name("human-documentation-style-allowlist.json")
FINDING_REASON = "unapproved-korean-narrative-ending"
ALLOWLIST_KEYS = {"path", "line", "reason", "text_sha256"}
SHA256 = re.compile(r"[0-9a-f]{64}")
ARCHIVE_NAVIGATION = {
    "docs/archive/README.md",
    "docs/archive/MANIFEST.md",
}

# A boundary can be an ordinary sentence mark, a quote, a Markdown table cell, or a
# label-like colon. It intentionally also admits whitespace so an authored sentence
# followed by another sentence on the same line cannot hide the first ending. An
# exact inline prompt followed by an explanatory postposition remains a literal
# boundary and therefore still requires an explicit allowlist entry.
BOUNDARY = (
    r"(?=(?:"
    r"[.!?。！？:：;；,，]*[)\]}>〉》」』’”'\"`]*[*_~]*(?:\s|\||$)"
    r"|[.!?。！？]+[)\]}>〉》」』’”'\"`]+[*_~]*(?:처럼|라고|이라는|라는|이라고)"
    r"(?:\s|[.!?。！？:：;；,，]|\||$)"
    r"))"
)
ENDING_TOKEN = re.compile(rf"[가-힣A-Za-z0-9_+-]+{BOUNDARY}")
INLINE_LINK_DESTINATION = re.compile(r"\]\((?:[^()]|\([^)]*\))*\)")
AUTOLINK = re.compile(r"<https?://[^>]+>")
PARTICLE_CONTEXT = re.compile(r"[가-힣]+(?:은|는|이|가|을|를|도|만|에서|에게|으로|로)\s+")

# These are grammatical shapes, not a list of example words. They cover explicit
# copulas/auxiliaries, past forms, connective negatives, action declaratives, and
# common adjective derivations. Generic `...다` is accepted only with structural
# clause evidence so ordinary nouns such as `바다` are not treated as conjugations.
UNAMBIGUOUS_DA_SHAPE = re.compile(
    r"(?:"
    r"(?:하|한|된|된다고|된다|되|이|있|없|않|같|맞|좋|크|작|높|낮|답|롭|스럽)다"
    r"|(?:했|됐|되었|이었|였|었|았)다"
    r"|[가-힣]*는다"
    r"|[가-힣]*ㄴ다"
    r")$"
)
MECHANICAL_M_NOMINALIZATION_SUFFIXES = ("남", "됨", "힘", "짐", "름")
NOMINALIZATION_NOUN_COLLISION_SUFFIXES = (
    "강남",
    "경남",
    "걸음",
    "구름",
    "기름",
    "다음",
    "다짐",
    "도움",
    "마음",
    "베트남",
    "여름",
    "얼음",
    "웃음",
    "이름",
    "전남",
    "처음",
    "충남",
    "하남",
    "흐름",
)


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    reason: str
    text_sha256: str


def is_candidate(name: str) -> bool:
    return name.endswith(CANDIDATE_SUFFIXES) or name.startswith(CANDIDATE_PREFIXES)


def inventory(root: Path) -> list[Path]:
    paths: list[Path] = []
    for directory, names, files in os.walk(root, followlinks=False):
        names[:] = sorted(name for name in names if name != ".git")
        for name in sorted(files):
            if is_candidate(name):
                path = Path(directory) / name
                relative = path.relative_to(root).as_posix()
                if relative.startswith("docs/archive/") and relative not in ARCHIVE_NAVIGATION:
                    continue
                paths.append(path)
    return sorted(paths, key=lambda path: path.relative_to(root).as_posix())


def _hangul_syllable_count(value: str) -> int:
    return sum("가" <= character <= "힣" for character in value)


def _final_consonant_index(character: str) -> int | None:
    if not "가" <= character <= "힣":
        return None
    return (ord(character) - ord("가")) % 28


def _is_attached_comparative(match: re.Match[str], line: str) -> bool:
    token = match.group(0)
    if not token.endswith("보다"):
        return False
    remainder = line[match.end():].lstrip()
    # `N보다 + following constituent` is an attached comparative particle, not a
    # declarative `...다` ending. Sentence-final `보다.` and compound verbs such as
    # `살펴보다.` have no following constituent and remain in scope.
    has_following_constituent = (
        re.match(r"(?:[*_~`]+)?[가-힣A-Za-z0-9]", remainder) is not None
    )
    if not has_following_constituent:
        return False
    if token != "보다":
        return True
    # Quotation and grouping delimiters split `“noun phrase”보다` into a standalone
    # `보다` token. The closed phrase immediately before it supplies the compared
    # noun; without that delimiter, standalone verb `보다` remains a finding.
    prefix = line[:match.start()].rstrip()
    return bool(prefix) and prefix[-1] in "\"'’”)]}〉》」』`"


def _is_mechanical_nominalization(token: str, line: str, start: int) -> bool:
    """Identify clause-like `~음/~ㅁ` rewrites without treating nouns as endings."""

    if token.endswith(NOMINALIZATION_NOUN_COLLISION_SUFFIXES):
        return False
    if token in {"있음", "없음"}:
        # Bare semantic nouns such as `API key 없음` remain concise. The productive
        # `verb + 수 있음/없음` construction is a mechanically nominalized clause.
        return re.search(r"[가-힣]+\s+수\s*$", line[:start]) is not None
    if token.endswith("음"):
        stem = token[:-1]
        return (
            bool(stem)
            and _hangul_syllable_count(stem) == len(stem)
            and _final_consonant_index(stem[-1]) not in {None, 0}
        )
    if token.endswith(MECHANICAL_M_NOMINALIZATION_SUFFIXES):
        stem = token[:-1]
        return bool(stem) and (
            _hangul_syllable_count(stem) == len(stem)
            or re.fullmatch(r"[A-Za-z][A-Za-z0-9_.+-]*", stem) is not None
        )
    return False


def _is_terminal_token(token: str, line: str, start: int) -> bool:
    remainder = line[start + len(token):]
    remainder = re.sub(r"^[)\]}>〉》」』’”'\"`*_~]+", "", remainder)
    stripped = remainder.lstrip()
    return not stripped or stripped[0] in ".!?。！？:：;；,，|"


def _is_narrative_token(token: str, line: str, start: int) -> bool:
    if token == "보다":
        return True
    if _is_mechanical_nominalization(token, line, start):
        return True
    if token.endswith(("했음", "됐음", "되었음", "않음", "이었음", "였음", "었음", "았음")):
        return True
    if token.endswith(("함", "임")):
        # A two-or-more-syllable stem distinguishes nominalized clauses (`사용함`,
        # `상태임`) from the most common inseparable nouns (`포함`, `책임`, `모임`).
        return _hangul_syllable_count(token[:-1]) >= 2
    if token.endswith("죠"):
        return True
    if token == "줘" and _is_terminal_token(token, line, start):
        return True
    if (
        token.endswith("해")
        and _hangul_syllable_count(token[:-1]) >= 2
        and _is_terminal_token(token, line, start)
    ):
        return PARTICLE_CONTEXT.search(line) is not None
    if re.search(r"(?:합니|됩니|입니|했어|됐어|해|돼|세|어|아|나|군|지)요$", token):
        return True
    if not token.endswith("다"):
        return False
    stem = token[:-1]
    if re.search(r"[A-Za-z0-9_.+-]$", stem):
        return True
    if UNAMBIGUOUS_DA_SHAPE.search(token):
        return True
    if not PARTICLE_CONTEXT.search(line) or _hangul_syllable_count(token) < 3:
        return False
    # Clause context can disambiguate otherwise unseen conjugations, but it is not
    # enough by itself: treating every final consonant before `다` as grammar makes
    # ordinary nouns such as `아젠다` findings. Admit only a vowel stem or the
    # productive present/past jongseong shapes (ㄴ/ㅆ) in an actual marked clause.
    final = _final_consonant_index(stem[-1]) if stem else None
    return final in {0, 4, 20}


def prose_findings(relative: str, text: str) -> list[Finding]:
    """Find authored Korean prose without blanket markup/literal exemptions.

    Fences, inline code, HTML comments, blockquotes, and protocol samples remain in
    scope. A Korean literal is exempt only when scan() matches its exact
    path+line+reason+line-digest allowlist entry. Structurally non-Korean code needs no
    exception because it cannot match a Korean ending.
    """

    findings: list[Finding] = []
    for number, raw_line in enumerate(text.splitlines(), 1):
        prose = INLINE_LINK_DESTINATION.sub("]()", raw_line)
        prose = AUTOLINK.sub("", prose)
        if any(
            not _is_attached_comparative(match, prose)
            and _is_narrative_token(match.group(0), prose, match.start())
            for match in ENDING_TOKEN.finditer(prose)
        ):
            findings.append(
                Finding(
                    path=relative,
                    line=number,
                    reason=FINDING_REASON,
                    text_sha256=hashlib.sha256(raw_line.encode("utf-8")).hexdigest(),
                )
            )
    return findings


def language(text: str) -> str:
    has_korean = bool(re.search(r"[가-힣]", text))
    has_english = bool(re.search(r"[A-Za-z]{3,}", text))
    if has_korean and has_english:
        return "mixed"
    if has_korean:
        return "korean"
    return "english"


def owner(relative: str) -> str:
    if relative.startswith((".agents/work/", ".omx/", "tests/work/", "target/")):
        return "runtime-generated"
    if relative.startswith("tests/fixtures/"):
        return "test-fixture"
    if relative == "LICENSE" or relative.startswith(("LICENSES/", "NOTICE")):
        return "protected-exact"
    if relative.startswith((".agents/", "harness/skills/")):
        return "ai-directive-skill"
    if relative.startswith("harness/template/"):
        return "canonical-template"
    return "project-source"


def disposition(relative: str, file_owner: str, file_language: str, findings: list[Finding]) -> str:
    if file_owner == "protected-exact":
        return "no-change-protected-exact"
    if file_owner == "runtime-generated":
        return "no-change-generated"
    if file_owner == "test-fixture":
        return "fixture-semantic-preserve"
    if file_language == "english":
        return "no-change-english"
    if findings:
        return "edit-project-source"
    return "no-change-compliant"


def tracked_paths(root: Path) -> set[str]:
    result = subprocess.run(
        ["git", "ls-files", "-z"], cwd=root, check=False, capture_output=True
    )
    if result.returncode:
        return set()
    return {value.decode("utf-8") for value in result.stdout.split(b"\0") if value}


def load_allowlist(path: Path = ALLOWLIST_PATH) -> list[dict[str, object]]:
    if not path.exists():
        return []
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, list):
        raise ValueError("style allowlist must be a JSON array")
    return value


def _literal_key(value: Finding | dict[str, object]) -> tuple[object, object, object]:
    if isinstance(value, Finding):
        return value.path, value.line, value.text_sha256
    return value["path"], value["line"], value["text_sha256"]


def _explicit_exception_reason(value: object) -> bool:
    if not isinstance(value, str):
        return False
    normalized = value.strip()
    if not normalized or normalized == FINDING_REASON:
        return False
    # A bare category does not explain why this exact byte-sensitive literal must
    # remain. Require a short human-readable justification rather than another code.
    return len(normalized) >= 12 and (" " in normalized or any("가" <= char <= "힣" for char in normalized))


def validate_allowlist(entries: list[dict[str, object]]) -> tuple[list[dict[str, object]], list[dict[str, str]]]:
    valid_entries: list[dict[str, object]] = []
    failures: list[dict[str, str]] = []
    seen: set[tuple[object, object, object]] = set()
    for entry in entries:
        path = str(entry.get("path", "<invalid>")) if isinstance(entry, dict) else "<invalid>"
        valid = (
            isinstance(entry, dict)
            and set(entry) == ALLOWLIST_KEYS
            and isinstance(entry.get("path"), str)
            and bool(str(entry.get("path", "")).strip())
            and isinstance(entry.get("line"), int)
            and not isinstance(entry.get("line"), bool)
            and int(entry["line"]) >= 1
            and _explicit_exception_reason(entry.get("reason"))
            and isinstance(entry.get("text_sha256"), str)
            and SHA256.fullmatch(str(entry["text_sha256"])) is not None
        )
        if not valid:
            failures.append({"path": path, "reason": "invalid-literal-allowlist-entry"})
            continue
        key = _literal_key(entry)
        if key in seen:
            failures.append({"path": path, "reason": "duplicate-literal-allowlist-entry"})
            continue
        seen.add(key)
        valid_entries.append(entry)
    return valid_entries, failures


def generated_relation(root: Path, relative: str, file_owner: str) -> tuple[str, str | None, bool]:
    """Return relation, exact producer when safely inferable, and orphan status."""

    if file_owner != "runtime-generated":
        return "self", relative, False
    if relative.startswith("target/"):
        return "cargo-generated", "Cargo.toml", not (root / "Cargo.toml").is_file()
    if relative.startswith(".omx/"):
        return "external-runtime-generated", None, False
    if relative.startswith(".agents/work/"):
        return "source-runtime-generated", None, False
    if relative.startswith("tests/work/"):
        suffix_relations = (
            ("/.hive/README.md", "harness/template/.hive/README.md"),
            ("/AGENTS.md", "harness/template/AGENTS.md.jinja"),
        )
        for suffix, producer in suffix_relations:
            if relative.endswith(suffix):
                return "canonical-template-generated", producer, not (root / producer).is_file()
        return "test-runtime-generated", None, False
    return "runtime-generated", None, False


def inventory_sha256(records: list[dict[str, object]]) -> str:
    """Hash inventory membership, not mutable contents, for usable drift detection."""

    digest = hashlib.sha256()
    for record in records:
        digest.update(str(record["path"]).encode("utf-8"))
        digest.update(b"\0")
    return digest.hexdigest()


def content_sha256(records: list[dict[str, object]]) -> str:
    digest = hashlib.sha256()
    for record in records:
        digest.update(str(record["path"]).encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(record["sha256"]).encode("ascii"))
        digest.update(b"\0")
    return digest.hexdigest()


def scan(root: Path, allowlist_path: Path = ALLOWLIST_PATH) -> dict[str, object]:
    root = root.resolve()
    tracked = tracked_paths(root)
    candidates = inventory(root)
    records: list[dict[str, object]] = []
    failures: list[dict[str, str]] = []
    all_findings: list[Finding] = []
    allowlist = load_allowlist(allowlist_path)
    valid_allowlist, allowlist_failures = validate_allowlist(allowlist)
    failures.extend(allowlist_failures)
    matched_allowlist: set[tuple[object, ...]] = set()
    for path in candidates:
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            failures.append({"path": relative, "reason": "symlink"})
            continue
        try:
            raw = path.read_bytes()
            if b"\0" in raw:
                raise ValueError("binary-nul")
            text = raw.decode("utf-8")
        except (OSError, UnicodeError, ValueError) as error:
            failures.append({"path": relative, "reason": str(error)})
            continue
        file_findings = prose_findings(relative, text)
        file_owner = owner(relative)
        file_language = language(text)
        relation, source_path, orphan = generated_relation(root, relative, file_owner)
        if orphan:
            failures.append({"path": relative, "reason": "orphan-generated-output"})
        unsuppressed: list[Finding] = []
        allowlist_by_literal = {_literal_key(entry): entry for entry in valid_allowlist}
        for finding in file_findings:
            key = _literal_key(finding)
            if key in allowlist_by_literal:
                matched_allowlist.add(key)
            else:
                unsuppressed.append(finding)
        file_disposition = disposition(relative, file_owner, file_language, unsuppressed)
        if file_disposition not in DISPOSITIONS:
            failures.append({"path": relative, "reason": "unreviewed"})
            continue
        if file_disposition == "edit-project-source":
            all_findings.extend(unsuppressed)
        records.append(
            {
                "path": relative,
                "sha256": hashlib.sha256(raw).hexdigest(),
                "bytes": len(raw),
                "language": file_language,
                "owner": file_owner,
                "tracked": relative in tracked,
                "source_relation": relation,
                "source_path": source_path,
                "source_exists": None if source_path is None else (root / source_path).is_file(),
                "disposition": file_disposition,
                "finding_count": len(file_findings),
                "unsuppressed_finding_count": len(unsuppressed),
            }
        )
    for entry in valid_allowlist:
        key = _literal_key(entry)
        if key not in matched_allowlist:
            failures.append({"path": str(entry["path"]), "reason": "stale-literal-allowlist-entry"})
    counts = {name: 0 for name in sorted(DISPOSITIONS)}
    for record in records:
        counts[str(record["disposition"])] += 1
    return {
        "schema_version": 2,
        "root": str(root),
        "inventory_count": len(candidates),
        "reviewed_count": len(records),
        "unreviewed_count": len(failures),
        "tracked_count": sum(bool(record["tracked"]) for record in records),
        "untracked_ignored_generated_count": sum(not bool(record["tracked"]) for record in records),
        "inventory_sha256": inventory_sha256(records),
        "content_sha256": content_sha256(records),
        "disposition_counts": counts,
        "allowlist_count": len(allowlist),
        "matched_allowlist_count": len(matched_allowlist),
        "finding_count": len(all_findings),
        "findings": [asdict(finding) for finding in all_findings],
        "failures": failures,
        "files": records,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--all", action="store_true", help="scan the full workspace inventory")
    parser.add_argument("--output", choices=("json",), default="json")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--allowlist", type=Path, default=ALLOWLIST_PATH)
    parser.add_argument("--write-audit", type=Path)
    parser.add_argument("--expect-inventory-sha256")
    parser.add_argument("--expect-inventory-from", type=Path, help="read inventory_sha256 from a previous JSON audit")
    arguments = parser.parse_args()
    if not arguments.all:
        parser.error("--all is required")
    expected_inventory = arguments.expect_inventory_sha256
    if arguments.expect_inventory_from:
        prior = json.loads(arguments.expect_inventory_from.read_text(encoding="utf-8"))
        prior_digest = prior.get("inventory_sha256")
        if not isinstance(prior_digest, str) or not SHA256.fullmatch(prior_digest):
            parser.error("--expect-inventory-from does not contain a valid inventory_sha256")
        if expected_inventory and expected_inventory != prior_digest:
            parser.error("inventory expectations disagree")
        expected_inventory = prior_digest
    report = scan(arguments.root, arguments.allowlist)
    inventory_drift = bool(expected_inventory and report["inventory_sha256"] != expected_inventory)
    report["expected_inventory_sha256"] = expected_inventory
    report["inventory_drift"] = inventory_drift
    rendered = json.dumps(report, ensure_ascii=False, indent=2) + "\n"
    if arguments.write_audit:
        arguments.write_audit.parent.mkdir(parents=True, exist_ok=True)
        arguments.write_audit.write_text(rendered, encoding="utf-8")
    sys.stdout.write(rendered)
    return 1 if report["unreviewed_count"] or report["finding_count"] or inventory_drift else 0


if __name__ == "__main__":
    raise SystemExit(main())
