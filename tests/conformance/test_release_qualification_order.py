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

    def test_v092_plan_separates_completed_release_from_future_work(self) -> None:
        text = (
            ROOT / "docs/plans/active/release-0.9.2-test-qualification.md"
        ).read_text(encoding="utf-8")
        for required in (
            "`0.9.2` 기능 기준: `2cec0377748874748d126b6b55e59975a3f20a02`",
            "`NAT-002–024`·`MRA-001–032`의 `0.9.3` 전용 branch",
            "`0.10.0` 유지: `N10-002–011`",
            "`codex/0.9.3-native-agents`",
            "`codex/release-0.9.2`",
            "`0.9.2-test.N`",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)


if __name__ == "__main__":
    unittest.main()
