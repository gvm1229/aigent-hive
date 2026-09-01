#!/usr/bin/env python3
"""Prompt refinement schema and semantic-validator conformance."""

from __future__ import annotations

import copy
import json
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.support.harness import (
    ACTION_RESULT_SCHEMA,
    Phase1CliTestCase,
    snapshot_tree,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
FIXTURE_ROOT = REPOSITORY_ROOT / "tests/fixtures/skills/prompt-refine"
INPUT_SCHEMA = json.loads(
    (
        REPOSITORY_ROOT / "schemas/prompt-refinement-input.schema.json"
    ).read_text(encoding="utf-8")
)
RESULT_SCHEMA = json.loads(
    (
        REPOSITORY_ROOT / "schemas/prompt-refinement-result.schema.json"
    ).read_text(encoding="utf-8")
)
LIFECYCLE_SCHEMA = json.loads(
    (
        REPOSITORY_ROOT / "schemas/prompt-refinement-lifecycle.schema.json"
    ).read_text(encoding="utf-8")
)
SELF_CONTAINED_RESULT_SCHEMA = copy.deepcopy(RESULT_SCHEMA)
SELF_CONTAINED_RESULT_SCHEMA["properties"]["preserved"] = copy.deepcopy(
    INPUT_SCHEMA["$defs"]["preservation"]
)
SELF_CONTAINED_RESULT_SCHEMA["$defs"] = {
    "textSet": copy.deepcopy(INPUT_SCHEMA["$defs"]["textSet"])
}


class Phase3PromptRefinementContract(Phase1CliTestCase):
    def read_fixture(self, name: str) -> dict[str, object]:
        value = json.loads((FIXTURE_ROOT / name).read_text(encoding="utf-8"))
        self.assertIsInstance(value, dict)
        return value

    def invoke_validate(
        self,
        request_name: str,
        result_name: str,
        *,
        cwd: Path | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        process = subprocess.run(
            [
                str(self.hive_binary),
                "prompt",
                "validate",
                "--request",
                str(FIXTURE_ROOT / request_name),
                "--result",
                str(FIXTURE_ROOT / result_name),
                "--output",
                "json",
            ],
            cwd=cwd or REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"prompt validator stdout must be one JSON object: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"])
        return process, result

    def invoke_approve(
        self,
        digest: str,
        *,
        target_host: str = "codex",
        confirm: bool = True,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        command = [
            str(self.hive_binary),
            "prompt",
            "approve",
            "--request",
            str(FIXTURE_ROOT / "valid-input.json"),
            "--result",
            str(FIXTURE_ROOT / "valid-result.json"),
            "--digest",
            digest,
            "--target-host",
            target_host,
        ]
        if confirm:
            command.append("--confirm-refined-prompt")
        command.extend(["--output", "json"])
        process = subprocess.run(
            command,
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        result = json.loads(process.stdout)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"])
        return process, result

    def test_valid_request_matches_input_schema(self) -> None:
        Draft202012Validator(
            INPUT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(self.read_fixture("valid-input.json"))

    def test_valid_result_matches_result_schema(self) -> None:
        Draft202012Validator(
            SELF_CONTAINED_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(self.read_fixture("valid-result.json"))

    def test_valid_refinement_preserves_meaning_and_authority(self) -> None:
        process, result = self.invoke_validate(
            "valid-input.json",
            "valid-result.json",
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["action"], "RefinePrompt")
        self.assertEqual(result["status"], "success")
        self.assertEqual(result["next_action"], "awaiting-approval")
        lifecycle = result["data"]
        self.assertIsInstance(lifecycle, dict)
        Draft202012Validator(
            LIFECYCLE_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(lifecycle)
        self.assertEqual(lifecycle["state"], "awaiting-approval")
        self.assertFalse(lifecycle["execution_authorized"])

    def test_refinement_missing_must_not_is_rejected(self) -> None:
        process, result = self.invoke_validate(
            "valid-input.json",
            "missing-must-not-result.json",
        )
        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(result["changed_paths"], [])

    def test_refinement_missing_user_authority_is_rejected(self) -> None:
        process, result = self.invoke_validate(
            "valid-input.json",
            "missing-authority-result.json",
        )
        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(result["changed_paths"], [])

    def test_refinement_altering_original_prompt_is_rejected(self) -> None:
        process, result = self.invoke_validate(
            "valid-input.json",
            "altered-original-result.json",
        )
        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(result["changed_paths"], [])

    def test_refine_and_run_without_explicit_intent_is_blocked(self) -> None:
        process, result = self.invoke_validate(
            "implicit-run-input.json",
            "implicit-run-result.json",
        )
        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["changed_paths"], [])

    def test_refine_and_run_with_explicit_intent_is_validated(self) -> None:
        process, result = self.invoke_validate(
            "explicit-run-input.json",
            "explicit-run-result.json",
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["status"], "success")
        self.assertEqual(result["next_action"], "host-owned-execution")
        lifecycle = result["data"]
        self.assertIsInstance(lifecycle, dict)
        self.assertEqual(lifecycle["state"], "authorized")
        self.assertTrue(lifecycle["execution_authorized"])

    def test_exact_followup_approval_authorizes_only_the_current_digest(self) -> None:
        digest = "sha256:4dc9439ca7e12adfccc9ecaddf0dc1636c6ff4bb572ed967fa004f943061c841"
        process, result = self.invoke_approve(digest)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.prompt-approved")
        self.assertEqual(result["next_action"], "host-owned-execution")
        data = result["data"]
        self.assertIsInstance(data, dict)
        self.assertEqual(data["state"], "authorized")
        self.assertEqual(data["refined_prompt_digest"], digest)
        self.assertEqual(data["target_host"], "codex")
        self.assertNotIn("refined_prompt", data)

    def test_stale_or_generic_followup_approval_is_blocked(self) -> None:
        for digest, confirm in (
            ("sha256:" + "0" * 64, True),
            (
                "sha256:4dc9439ca7e12adfccc9ecaddf0dc1636c6ff4bb572ed967fa004f943061c841",
                False,
            ),
        ):
            with self.subTest(digest=digest, confirm=confirm):
                process, result = self.invoke_approve(digest, confirm=confirm)
                self.assertEqual(process.returncode, 3, process.stderr)
                self.assertEqual(result["status"], "blocked")
                self.assertEqual(result["changed_paths"], [])

    def test_refine_and_run_missing_user_authority_is_rejected(self) -> None:
        process, result = self.invoke_validate(
            "explicit-run-input.json",
            "explicit-run-missing-authority-result.json",
        )
        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(result["changed_paths"], [])

    def test_multiple_ambiguity_questions_are_rejected(self) -> None:
        process, result = self.invoke_validate(
            "two-questions-input.json",
            "two-questions-result.json",
        )
        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(result["changed_paths"], [])

    def test_refine_only_result_has_no_side_effects(self) -> None:
        fixture = self.read_fixture("valid-result.json")
        self.assertEqual(fixture["mode"], "refine-only")
        self.assertFalse(fixture["execution_authorized"])
        self.assertEqual(fixture["project_reads"], [])
        self.assertEqual(
            set(fixture["side_effects"].values()),
            {False},
        )

    def test_provider_neutral_result_has_no_host_specific_surface(
        self,
    ) -> None:
        request = self.read_fixture("valid-input.json")
        result = self.read_fixture("valid-result.json")
        self.assertIsNone(request["target_host"])
        refined = str(result["refined_prompt"]).casefold()
        for token in (
            ".codex",
            ".claude",
            ".agents",
            "omx",
            "omc",
            "codex ",
            "claude ",
            "antigravity ",
        ):
            with self.subTest(token=token):
                self.assertNotIn(token, refined)

    def test_rejected_refinement_does_not_mutate_project(self) -> None:
        consumer = self.work_root / "consumer"
        consumer.mkdir()
        (consumer / "sentinel.bin").write_bytes(b"user bytes\x00\xff\n")
        before = snapshot_tree(consumer)

        process, result = self.invoke_validate(
            "valid-input.json",
            "missing-must-not-result.json",
            cwd=consumer,
        )

        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(consumer), before)


if __name__ == "__main__":
    import unittest

    unittest.main()
