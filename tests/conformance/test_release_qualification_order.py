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

    def test_release_gate_requires_the_exact_prior_patch_project_base(self) -> None:
        text = (ROOT / "scripts/check-release-version.sh").read_text(encoding="utf-8")
        for required in (
            "missing frozen full project base for prior patch release",
            "prior patch base inventory differs",
            "prior patch base bytes differ",
            "git", "ls-tree", "v{previous}", "harness/template",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)

    def test_candidate_workflow_binds_project_base_coverage_to_the_artifact_set(self) -> None:
        text = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        for required in (
            "Generate digest-bound project upgrade coverage report",
            "scripts/check-project-base-coverage.py",
            "harness/release/$PRODUCT_VERSION/migration-table.json",
            "release-project-base-coverage.json",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)

    def test_public_test_requires_separate_publication_and_full_tag_history(self) -> None:
        directive = re.sub(
            r"\s+",
            " ",
            (ROOT / ".agents/directives/03-workflow.md").read_text(encoding="utf-8"),
        )
        candidate = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        publication = (ROOT / ".github/workflows/release-publish.yml").read_text(
            encoding="utf-8"
        )
        for required in (
            "private artifact-generation result only",
            "separate publication workflow succeeds",
            "independent registry and GitHub Release checks",
            "fetch-depth: 0",
        ):
            with self.subTest(required=required):
                self.assertIn(required, directive)
        self.assertIn("fetch-depth: 0", candidate)
        self.assertIn("fetch-depth: 0", publication)
        self.assertIn("candidate_run_id", publication)

    def test_runtime_qualification_keeps_history_for_release_version_checks(self) -> None:
        runtime = (ROOT / ".github/workflows/release-runtime.yml").read_text(
            encoding="utf-8"
        )
        self.assertEqual(runtime.count("actions/checkout@"), 3)
        self.assertEqual(runtime.count("fetch-depth: 0"), 3)
        self.assertIn("scripts/check-release-version.sh \"$version\"", runtime)
        self.assertIn("bash scripts/check-release-version.sh $version", runtime)

    def test_product_changes_remain_bound_to_the_full_risk_verification_path(self) -> None:
        text = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        for required in (
            "Classify tracked change risk",
            "scope=product",
            "cargo test --workspace --all-targets --all-features --locked",
            "Run complete named conformance inventory",
            "Require the risk-matched verification result",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)

    def test_v092_plan_releases_completed_scope_and_defers_future_work(self) -> None:
        text = (
            ROOT / "docs/archive/plans/releases/0.9.2/release-0.9.2-test-qualification.md"
        ).read_text(encoding="utf-8")
        for required in (
            "`2cec0377748874748d126b6b55e59975a3f20a02`",
            "`NAT-002–024`·`MRA-001–032`의 `0.9.3`",
            "`N10-002–011`의 `0.10.0-test`",
            "`codex/release-0.9.2`",
            "`0.9.2-test.N`",
            "root·번역 README, 설치 안내, 공개 HTML",
            "QA contributor 등록 뒤 유지보수자의 별도 명시적 승인 전 금지",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)


if __name__ == "__main__":
    unittest.main()
