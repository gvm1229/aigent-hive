from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CHECKER = ROOT / "scripts/check-release-notes.py"


def note(*, english_compatibility_id: str = "COMPAT-01", korean_compatibility_id: str = "COMPAT-01") -> str:
    return f"""# Aigent Hive 0.9.4

## English

### Scope

- [SCOPE-01] Add the verified Hive projection validation contract.

### Compatibility

- [{english_compatibility_id}] Keep Markdown as the canonical knowledge source.

### Verification

- [VERIFY-01] Run the release candidate checks on Windows x64.

### Publication

- [PUBLISH-01] Publish one numbered test release before stable publication.

## 한국어

### 범위

- [SCOPE-01] 검증된 Hive 투영 검사 계약 추가.

### 호환성

- [{korean_compatibility_id}] Markdown을 지식 정본으로 유지.

### 검증

- [VERIFY-01] Windows x64에서 출시 후보 검사 실행.

### 게시

- [PUBLISH-01] 정식 게시 전에 번호가 있는 시험판 한 개 게시.
"""


class ReleaseNoteContract(unittest.TestCase):
    def run_checker(self, content: str) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "0.9.4.md"
            path.write_text(content, encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(CHECKER), "--version", "0.9.4", "--path", str(path)],
                cwd=ROOT,
                check=False,
                capture_output=True,
                text=True,
            )

    def test_accepts_english_first_equivalent_bilingual_release_note(self) -> None:
        result = self.run_checker(note())
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_different_required_fact_ids(self) -> None:
        result = self.run_checker(note(korean_compatibility_id="COMPAT-02"))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("fact IDs differ", result.stderr)

    def test_rejects_non_simplified_english(self) -> None:
        result = self.run_checker(note().replace("Add the verified", "We can easily add the verified"))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not ASD-STE100 concise", result.stderr)

    def test_rejects_ordinary_english_in_korean_section(self) -> None:
        result = self.run_checker(note().replace("검증된 Hive 투영 검사 계약 추가.", "the Hive 투영 검사 계약 추가."))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("ordinary English prose", result.stderr)

    def test_publication_workflow_checks_the_canonical_release_note(self) -> None:
        workflow = (ROOT / ".github/workflows/release-publish.yml").read_text(encoding="utf-8")
        self.assertIn("python3 scripts/check-release-notes.py", workflow)
        self.assertIn('--version "$PRODUCT_VERSION"', workflow)
        self.assertIn('--path "$release_notes"', workflow)
