#!/usr/bin/env python3
"""Public Korean test acceptance must install and qualify exact registry bytes."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = ROOT / ".github/workflows/public-test-acceptance.yml"
REGISTERED_WORKFLOW = ROOT / ".github/workflows/release-runtime.yml"
SCRIPT = ROOT / "scripts/qualify-korean-public-test.py"
VECTOR_ONBOARDING_SCRIPT = ROOT / "scripts/qualify-vector-onboarding-public-test.py"


class PublicTestAcceptanceContract(unittest.TestCase):
    def test_workflow_installs_exact_public_test_on_three_platforms(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")
        for required in (
            "windows-latest",
            "macos-15",
            "ubuntu-24.04",
            'git diff --name-only "$tag_commit"..HEAD',
            'aigent-hive@$PACKAGE_VERSION',
            "dist-tags.test",
            "dist-tags.latest",
            'test "$(npm view aigent-hive \'dist-tags.latest\')" = "0.9.5"',
            "qualify-korean-public-test.py",
            "qualify-vector-onboarding-public-test.py",
            '--package-version "$PACKAGE_VERSION"',
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)
        self.assertNotIn("channel: stable", text)
        self.assertIn("workflow_call:", text)

    def test_registered_runtime_workflow_calls_the_public_test_gate(self) -> None:
        text = REGISTERED_WORKFLOW.read_text(encoding="utf-8")
        for required in (
            "public_test_product_version",
            "public_test_package_version",
            "public_test_release_date",
            "uses: ./.github/workflows/public-test-acceptance.yml",
            "inputs.public_test_package_version != ''",
            "inputs.public_test_package_version == ''",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)

    def test_qualifier_has_bounded_preservation_and_pack_rollback_checks(self) -> None:
        text = SCRIPT.read_text(encoding="utf-8")
        for required in (
            '"korean", "inspect"',
            '"korean", "verify"',
            '"korean", "sanitize"',
            '"korean", "pack", "check"',
            '"korean", "pack", "preview"',
            '"korean", "pack", "activate"',
            '"korean", "pack", "rollback"',
            '"provider_api_calls": 0',
            '"api_keys_read": 0',
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)

    def test_qualifier_imports_without_third_party_dependencies(self) -> None:
        spec = importlib.util.spec_from_file_location("korean_public", SCRIPT)
        self.assertIsNotNone(spec)
        self.assertIsNotNone(spec.loader)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertTrue(callable(module.qualify))

    def test_vector_onboarding_qualifier_keeps_one_question_and_a_fixed_scope(self) -> None:
        text = VECTOR_ONBOARDING_SCRIPT.read_text(encoding="utf-8")
        for required in (
            '"setup", "feature"',
            '"claim"',
            '"answer", "--answer", "yes"',
            '"answer", "--answer", "no"',
            '"project-private"',
            '"confidential"',
            '"setup_request_digest"',
            '"session"',
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)
        spec = importlib.util.spec_from_file_location("vector_onboarding_public", VECTOR_ONBOARDING_SCRIPT)
        self.assertIsNotNone(spec)
        self.assertIsNotNone(spec.loader)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertTrue(callable(module.qualify))


if __name__ == "__main__":
    unittest.main()
