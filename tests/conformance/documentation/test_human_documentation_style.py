from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/check-human-documentation-style.py"
SPEC = importlib.util.spec_from_file_location("human_documentation_style", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class HumanDocumentationStyleTest(unittest.TestCase):
    def test_archive_history_is_excluded_but_archive_navigation_is_checked(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs/archive/state").mkdir(parents=True)
            (root / "docs/archive/README.md").write_text("# Archive\n", encoding="utf-8")
            (root / "docs/archive/MANIFEST.md").write_text("# Manifest\n", encoding="utf-8")
            (root / "docs/archive/state/old.md").write_text(
                "기능을 사용합니다.\n", encoding="utf-8"
            )

            inventory = [path.relative_to(root).as_posix() for path in MODULE.inventory(root)]

            self.assertIn("docs/archive/README.md", inventory)
            self.assertIn("docs/archive/MANIFEST.md", inventory)
            self.assertNotIn("docs/archive/state/old.md", inventory)

    def test_inventory_reads_hidden_ignored_and_generated_candidates(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in ("README.md", ".hidden/NOTE.txt", "tests/work/out.md"):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("간결한 설명\n", encoding="utf-8")
            self.assertEqual(
                [path.relative_to(root).as_posix() for path in MODULE.inventory(root)],
                [".hidden/NOTE.txt", "README.md", "tests/work/out.md"],
            )

    def test_literal_looking_korean_prose_is_not_blanket_exempted(self) -> None:
        text = """본문은 정리합니다.\n`예시는 정리합니다.`\n```text\n프로토콜은 유지합니다.\n```\n<!-- 상태가 바뀌었다. -->\n[링크](https://example.test/정리합니다.)\n"""
        self.assertEqual(
            [finding.line for finding in MODULE.prose_findings("README.md", text)],
            [1, 2, 4, 6],
        )

    def test_general_declarative_and_conversational_shapes_are_checked(self) -> None:
        text = "> 이 계약은 provider-neutral harness다.\n상태가 바뀌었다.\n키를 저장하지 않는다.\n설정이 아름답다.\n다음 단계에서 검증해요.\n그렇죠?\n"
        self.assertEqual(
            [finding.line for finding in MODULE.prose_findings("README.md", text)],
            [1, 2, 3, 4, 5, 6],
        )

    def test_conversational_imperatives_are_checked_without_lexical_collisions(self) -> None:
        prohibited = (
            "문서를 보여 줘.\n"
            "이 기능을 사용해.\n"
            "우회를 계속해.\n"
            "`이 session에서 사용량 가드를 우회하고 계속해.`처럼 명시 필요.\n"
        )
        self.assertEqual(
            [finding.line for finding in MODULE.prose_findings("README.md", prohibited)],
            [1, 2, 3, 4],
        )
        allowed = "동해\n사용자 오해\n기능 사용 요청\n설명을 통해 결과 확인\nanswers로 전달해 migration 수행\n"
        self.assertEqual(MODULE.prose_findings("README.md", allowed), [])

    def test_mechanical_nominalized_endings_are_checked(self) -> None:
        text = """기능 사용함
업데이트 완료했음.
계약 구현됐음!
API key를 저장하지 않음
현재 상태임
Status는 INDETERMINATE다.
문서를 읽음.
작업이 끝남.
연결이 닫힘.
설정 값을 가짐.
정책을 따름.
compile됨.
검증할 수 있음.
검증할 수 없음.
"""
        self.assertEqual(
            [finding.line for finding in MODULE.prose_findings("README.md", text)],
            list(range(1, 15)),
        )

    def test_semantic_noun_phrases_and_morphological_collisions_are_allowed(self) -> None:
        text = """검증 필요
API key 없음
상태 있음
업데이트 완료
푸른 바다.
회의 아젠다.
업무 책임
정기 모임
범위 포함
구성 개요
사용자 마음
첫걸음
얼음
처음
다음
이름
도움
웃음
강남
여름
구름
힘
짐
"""
        self.assertEqual(MODULE.prose_findings("README.md", text), [])

    def test_code_identifiers_and_structural_literals_do_not_create_findings(self) -> None:
        text = """`README.md`
`compile()`
status=INDETERMINATE
{"state":"done"}
docs/plans/PLAN.md
"""
        self.assertEqual(MODULE.prose_findings("README.md", text), [])

    def test_attached_comparative_boda_is_not_a_da_ending(self) -> None:
        text = """Session은 weekly의 low, malformed 또는 duplicate 상태보다 우선. Session
기본값보다 명시 설정 우선
외부 blocker는 “계속 실행”보다 우선.
외부 blocker는 (계속 실행)보다 우선.
"""
        self.assertEqual(MODULE.prose_findings("README.md", text), [])

    def test_sentence_final_boda_verbs_remain_findings(self) -> None:
        text = "결과를 자세히 보다.\n결과를 다시 살펴보다.\n"
        self.assertEqual(
            [finding.line for finding in MODULE.prose_findings("README.md", text)],
            [1, 2],
        )

    def test_punctuation_quotes_tables_and_colons_are_boundaries(self) -> None:
        text = "“기능 사용함.”\n| 결과 | 구현됐음 |\n상태: 저장하지 않음\n완료했음: 후속 검증\n두 문장을 사용한다. 다음 항목: 확인\n"
        self.assertEqual(
            [finding.line for finding in MODULE.prose_findings("README.md", text)],
            [1, 2, 3, 4, 5],
        )

    def test_each_reviewed_file_receives_exactly_one_valid_disposition(self) -> None:
        cases = (
            ("LICENSE", "protected-exact", "english", [], "no-change-protected-exact"),
            ("target/note.txt", "runtime-generated", "english", [], "no-change-generated"),
            ("tests/fixtures/a.md", "test-fixture", "mixed", [], "fixture-semantic-preserve"),
            ("README.md", "project-source", "english", [], "no-change-english"),
            ("README.md", "project-source", "mixed", [], "no-change-compliant"),
        )
        for path, owner, language, findings, expected in cases:
            with self.subTest(path=path, expected=expected):
                actual = MODULE.disposition(path, owner, language, findings)
                self.assertIn(actual, MODULE.DISPOSITIONS)
                self.assertEqual(actual, expected)

    def test_literal_allowlist_requires_exact_path_line_reason_and_digest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            line = "프로토콜은 유지합니다."
            (root / "README.md").write_text(f"```text\n{line}\n```\n", encoding="utf-8")
            allowlist = root / "allowlist.json"
            entry = {
                "path": "README.md",
                "line": 2,
                "reason": "protocol sample required for byte-exact compatibility",
                "text_sha256": hashlib.sha256(line.encode()).hexdigest(),
            }
            allowlist.write_text(json.dumps([entry]), encoding="utf-8")
            report = MODULE.scan(root, allowlist)
            self.assertEqual(report["finding_count"], 0)
            self.assertEqual(report["matched_allowlist_count"], 1)
            self.assertEqual(report["failures"], [])

    def test_allowlist_rejects_missing_generic_stale_and_changed_reason_duplicates(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            line = "프로토콜은 변경합니다."
            (root / "README.md").write_text(line + "\n", encoding="utf-8")
            digest = hashlib.sha256(line.encode()).hexdigest()
            allowlist = root / "allowlist.json"
            stale = {
                "path": "README.md",
                "line": 1,
                "reason": "legacy protocol byte retained for compatibility",
                "text_sha256": "0" * 64,
            }
            first = {
                "path": "README.md",
                "line": 1,
                "reason": "protocol negative example retained for guidance",
                "text_sha256": digest,
            }
            changed_reason = {
                **first,
                "reason": "changed explanation cannot create a second exception",
            }
            generic = {**first, "line": 2, "reason": MODULE.FINDING_REASON}
            missing = {key: value for key, value in first.items() if key != "reason"}
            allowlist.write_text(
                json.dumps([stale, first, changed_reason, generic, missing]),
                encoding="utf-8",
            )
            report = MODULE.scan(root, allowlist)
            reasons = [failure["reason"] for failure in report["failures"]]
            self.assertIn("duplicate-literal-allowlist-entry", reasons)
            self.assertEqual(reasons.count("invalid-literal-allowlist-entry"), 2)
            self.assertIn("stale-literal-allowlist-entry", reasons)
            self.assertEqual(report["finding_count"], 0)
            self.assertEqual(report["matched_allowlist_count"], 1)

    def test_changed_explicit_justification_still_matches_exact_literal_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            line = "`API key를 요청하거나 저장하지 않는다.`"
            (root / "README.md").write_text(line + "\n", encoding="utf-8")
            allowlist = root / "allowlist.json"
            base = {
                "path": "README.md",
                "line": 1,
                "text_sha256": hashlib.sha256(line.encode()).hexdigest(),
            }
            for reason in (
                "directive negative example preserved verbatim",
                "exact prohibited form demonstrates the required rewrite",
            ):
                with self.subTest(reason=reason):
                    allowlist.write_text(json.dumps([{**base, "reason": reason}]), encoding="utf-8")
                    report = MODULE.scan(root, allowlist)
                    self.assertEqual(report["finding_count"], 0)
                    self.assertEqual(report["matched_allowlist_count"], 1)
                    self.assertEqual(report["failures"], [])

    def test_exact_bad_forms_and_unseen_grammar_stems_are_checked(self) -> None:
        text = """Aigent Hive는 provider-neutral 로컬 agent harness다.
Product version은 0.7.0이다.
Release 계약이 구현됐다.
API key를 요청하거나 저장하지 않는다.
이 기능을 사용합니다.
다음 단계에서 검증해요.
검증이 필요합니다.
업데이트가 완료되었습니다.
Release 계약이 구현됐음.
API key를 요청하거나 저장하지 않음.
새 결과를 집계한다.
기록이 누락되었다.
구성이 단순해졌다.
"""
        self.assertEqual(
            [finding.line for finding in MODULE.prose_findings("README.md", text)],
            list(range(1, 14)),
        )

    def test_symlink_and_safely_inferred_generated_orphan_are_failures(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target.md"
            target.write_text("text\n", encoding="utf-8")
            try:
                (root / "link.md").symlink_to("target.md")
            except OSError as error:
                self.skipTest(f"symlink creation is unavailable: {error}")
            output = root / "tests/work/render/AGENTS.md"
            output.parent.mkdir(parents=True)
            output.write_text("generated\n", encoding="utf-8")
            allowlist = root / "allowlist.json"
            allowlist.write_text("[]", encoding="utf-8")
            report = MODULE.scan(root, allowlist)
            reasons = [item["reason"] for item in report["failures"]]
            self.assertIn("symlink", reasons)
            self.assertIn("orphan-generated-output", reasons)
            producer = root / "harness/template/AGENTS.md.jinja"
            producer.parent.mkdir(parents=True)
            producer.write_text("template\n", encoding="utf-8")
            report = MODULE.scan(root, allowlist)
            reasons = [item["reason"] for item in report["failures"]]
            self.assertNotIn("orphan-generated-output", reasons)
            generated = next(item for item in report["files"] if item["path"].endswith("AGENTS.md"))
            self.assertEqual(generated["source_relation"], "canonical-template-generated")
            self.assertEqual(generated["source_path"], "harness/template/AGENTS.md.jinja")
            self.assertTrue(generated["source_exists"])

    def test_inventory_digest_ignores_content_edits_but_detects_path_drift(self) -> None:
        records = [{"path": "README.md", "sha256": "a" * 64}]
        digest = MODULE.inventory_sha256(records)
        self.assertEqual(
            digest,
            MODULE.inventory_sha256([{"path": "README.md", "sha256": "b" * 64}]),
        )
        self.assertNotEqual(
            digest,
            MODULE.inventory_sha256(records + [{"path": "docs/a.md", "sha256": "c" * 64}]),
        )

    def test_cli_compares_inventory_with_previous_audit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "README.md").write_text("concise text\n", encoding="utf-8")
            allowlist = root / "allowlist.json"
            allowlist.write_text("[]", encoding="utf-8")
            audit = root / "audit.json"
            base = [sys.executable, str(SCRIPT), "--all", "--root", str(root), "--allowlist", str(allowlist)]
            first = subprocess.run(base + ["--write-audit", str(audit)], check=False, capture_output=True, text=True)
            self.assertEqual(first.returncode, 0, first.stdout + first.stderr)
            (root / "README.md").write_text("changed concise text\n", encoding="utf-8")
            stable = subprocess.run(base + ["--expect-inventory-from", str(audit)], check=False, capture_output=True, text=True)
            self.assertEqual(stable.returncode, 0, stable.stdout + stable.stderr)
            (root / "NEW.md").write_text("new\n", encoding="utf-8")
            drift = subprocess.run(base + ["--expect-inventory-from", str(audit)], check=False, capture_output=True, text=True)
            self.assertEqual(drift.returncode, 1)
            self.assertTrue(json.loads(drift.stdout)["inventory_drift"])


if __name__ == "__main__":
    unittest.main()
