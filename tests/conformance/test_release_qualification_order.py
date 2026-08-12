#!/usr/bin/env python3
"""Stable publication must remain behind numbered public-test acceptance."""

from __future__ import annotations

import unittest
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[2]


class ReleaseQualificationOrderContract(unittest.TestCase):
    def test_workflow_forbids_stable_as_test(self) -> None:
        text = re.sub(
            r"\s+",
            " ",
            (ROOT / ".agents/directives/03-workflow.md").read_text(encoding="utf-8"),
        )
        for required in (
            "Never publish or install a stable version as exploratory",
            "uniquely numbered public test version",
            "Any product, packaging, installer, metadata, or acceptance fix invalidates",
            "Stable publication cannot create missing qualification evidence",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)

    def test_plan_gate_requires_zero_in_scope_items_and_public_test(self) -> None:
        text = re.sub(
            r"\s+",
            " ",
            (ROOT / ".agents/directives/04-documentation-state.md").read_text(
                encoding="utf-8"
            ),
        )
        for required in (
            "stable publication blocked while any active in-scope checklist item is incomplete",
            "uniquely numbered public test version",
            "A post-test change resets the affected acceptance item",
            "stable channel supplied the first or only evidence",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)

    def test_v092_plan_releases_completed_scope_and_defers_future_work(self) -> None:
        text = (
            ROOT / "docs/plans/active/release-0.9.2-test-qualification.md"
        ).read_text(encoding="utf-8")
        for required in (
            "`2cec0377748874748d126b6b55e59975a3f20a02`",
            "`NAT-002–024`·`MRA-001–032`의 `0.9.3`",
            "`N10-002–011`의 `0.10.0-test`",
            "`codex/release-0.9.2`",
            "`0.9.2-test.N`",
            "root·번역 README, 설치 안내, 공개 HTML",
            "QA contributor 추가 지시 뒤 유지보수자의 별도 명시적 승인 전 금지",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)


if __name__ == "__main__":
    unittest.main()
