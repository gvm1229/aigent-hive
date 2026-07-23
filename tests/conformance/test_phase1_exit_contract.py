#!/usr/bin/env python3
"""ActionResult exit mapping and stable CLI domain-code conformance."""

from __future__ import annotations

import os
import subprocess
import unittest

from jsonschema import Draft202012Validator

from tests.conformance.phase1_support import (
    ACTION_RESULT_SCHEMA,
    FIXTURE_ROOT,
    Phase1CliTestCase,
    snapshot_tree,
)


class Phase1ActionResultSchemaConformance(unittest.TestCase):
    pass


EXIT_STATUS_CASES = {
    0: "success",
    2: "error",
    3: "conflict",
    4: "unsupported",
    5: "verification-failed",
    10: "error",
}


def make_exit_schema_test(exit_code: int, status: str):
    def test(self: Phase1ActionResultSchemaConformance) -> None:
        instance = {
            "schema_version": 1,
            "action": "SetupHarness",
            "status": status,
            "exit_code": exit_code,
            "code": f"hive.fixture-exit-{exit_code}",
            "message": f"fixture exit {exit_code}",
            "changed_paths": [],
            "evidence": [],
            "next_action": None,
        }

        Draft202012Validator(ACTION_RESULT_SCHEMA).validate(instance)

    test.__name__ = f"test_exit_{exit_code}_maps_to_{status.replace('-', '_')}"
    return test


for mapped_exit, mapped_status in EXIT_STATUS_CASES.items():
    setattr(
        Phase1ActionResultSchemaConformance,
        f"test_exit_{mapped_exit}_maps_to_{mapped_status.replace('-', '_')}",
        make_exit_schema_test(mapped_exit, mapped_status),
    )


class Phase1CliExitConformance(Phase1CliTestCase):
    def test_apply_success_uses_stable_success_code(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(target)

        self.assertEqual(process.returncode, 0)
        self.assertEqual(result["status"], "success")
        self.assertEqual(result["code"], "hive.setup-complete")

    def test_invalid_input_uses_stable_exit_two_code_before_write(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            extra_arguments=("--unknown-setup-option",),
        )

        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["status"], "error")
        self.assertEqual(result["code"], "hive.setup-invalid-input")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def test_conflict_uses_stable_exit_three_code_before_write(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        agents = target / "AGENTS.md"
        agents.write_bytes(
            (FIXTURE_ROOT / "conflicting-agents.md").read_bytes()
        )
        before = snapshot_tree(target)

        process, result = self.invoke_setup(target)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(result["status"], "conflict")
        self.assertEqual(result["code"], "hive.setup-conflict")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def test_verification_failure_uses_stable_exit_five_code(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-approved-skill.yml",
        )
        self.assertEqual(first_process.returncode, 0)
        ledger = target / ".hive/config/approved-skills.yml"
        ledger.write_text("skills: [malformed\n", encoding="utf-8")
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-approved-skill.yml",
            mode="--validate",
        )

        self.assertEqual(process.returncode, 5)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(result["code"], "hive.setup-verification-failed")
        self.assertEqual(snapshot_tree(target), before)

    def test_atomic_activation_permission_failure_uses_stable_exit_ten_code(
        self,
    ) -> None:
        if os.name == "nt":
            self.skipTest(
                "Windows ACLs do not provide a portable chmod write-failure injection"
            )
        target = self.work_root / "consumer"
        target.mkdir()
        target.chmod(0o500)
        before = snapshot_tree(target)

        try:
            process, result = self.invoke_setup(target)
        finally:
            target.chmod(0o700)

        self.assertEqual(process.returncode, 10)
        self.assertEqual(result["status"], "error")
        self.assertEqual(result["code"], "hive.internal-error")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def test_unknown_top_level_action_performs_no_project_write(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        (target / "sentinel.txt").write_bytes(b"user bytes\n")
        before = snapshot_tree(target)

        process = subprocess.run(
            [str(self.hive_binary), "unknown-action"],
            cwd=target,
            check=False,
            text=True,
            capture_output=True,
        )

        self.assertEqual(process.returncode, 2)
        self.assertEqual(snapshot_tree(target), before)

    def test_phase_one_does_not_claim_stage_zero_semantic_routing(self) -> None:
        fixture_contract = (
            FIXTURE_ROOT / "README.md"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "Stage 0 semantic Skill routing is deferred to Phase 3",
            fixture_contract,
        )
