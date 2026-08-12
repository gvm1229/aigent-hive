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

    def test_v092_plan_defers_only_notion_candidate(self) -> None:
        text = (
            ROOT / "docs/plans/active/release-0.9.2-test-qualification.md"
        ).read_text(encoding="utf-8")
        self.assertIn("제외: `N10-002–011`", text)
        self.assertIn("`N10-002–011` 외 활성 계획 미완료 `0건`", text)
        self.assertIn("`0.9.2-test.1`", text)


if __name__ == "__main__":
    unittest.main()
