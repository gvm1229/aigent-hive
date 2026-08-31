#!/usr/bin/env python3
"""Public vector-search onboarding commands and saved-answer boundaries."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.support.harness import (
    ACTION_RESULT_SCHEMA,
    Phase1CliTestCase,
    read_yaml,
)


ROOT = Path(__file__).resolve().parents[3]


class VectorOnboardingContract(Phase1CliTestCase):
    """Exercise the one-time user answer through the installed CLI boundary."""

    def invoke_feature(
        self,
        action: str,
        *extra: str,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        command = [
            str(self.hive_binary),
            "setup",
            "feature",
            action,
            "--id",
            "vector-search",
            "--user-root",
            str(self.setup_user_root),
            *extra,
            "--output",
            "json",
        ]
        process = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
        )
        result = json.loads(process.stdout)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        return process, result

    def assert_success(self, result: dict[str, object]) -> dict[str, object]:
        self.assertEqual(result["status"], "success", result)
        data = result.get("data")
        self.assertIsInstance(data, dict, result)
        return data

    def test_claim_yes_prompt_and_explicit_no_keep_setup_separate(self) -> None:
        setup_path = self.setup_user_root / ".hive/config/user-setup.yml"
        setup_before = setup_path.read_bytes()

        claim = self.assert_success(self.invoke_feature("claim")[1])
        self.assertTrue(claim["question_required"])
        self.assertTrue(claim["question_pending"])
        self.assertTrue(claim["question_claim_active"])

        concurrent = self.assert_success(self.invoke_feature("claim")[1])
        self.assertFalse(concurrent["question_required"])
        self.assertTrue(concurrent["question_pending"])

        yes = self.assert_success(
            self.invoke_feature("answer", "--answer", "yes")[1]
        )
        self.assertEqual(yes["answer"], "yes")
        self.assertIsInstance(yes["answered_at_unix"], int)
        self.assertFalse(yes["question_claim_active"])
        saved = read_yaml(
            self.setup_user_root / ".hive/config/user-feature-answers.yml"
        )
        self.assertEqual(saved["vector_search"], "yes")
        self.assertIn("introduced_in", saved)
        self.assertIn("answered_at_unix", saved)
        self.assertNotIn("question_claimed_at_unix", saved)

        prompt = self.assert_success(self.invoke_feature("prompt")[1])
        self.assertEqual(prompt["scope_collection_ids"], ["user-root"])
        self.assertRegex(prompt["setup_request_digest"], r"^sha256:[0-9a-f]{64}$")
        self.assertIn(prompt["setup_request_digest"], prompt["prompt"])
        self.assertIn("project-private", prompt["prompt"])
        self.assertIn("confidential", prompt["prompt"])
        self.assertEqual(setup_path.read_bytes(), setup_before)

        no = self.assert_success(
            self.invoke_feature("answer", "--answer", "no")[1]
        )
        self.assertEqual(no["answer"], "no")
        status = self.assert_success(self.invoke_feature("status")[1])
        self.assertEqual(status["answer"], "no")
        self.assertFalse(status["question_pending"])
        self.assertFalse(status["question_required"])

    def test_expired_claim_allows_a_later_session_without_persisting_identity(self) -> None:
        answers = self.setup_user_root / ".hive/config/user-feature-answers.yml"
        answers.write_text(
            "schema_version: 1\nquestion_claimed_at_unix: 0\n",
            encoding="utf-8",
        )

        claimed = self.assert_success(self.invoke_feature("claim")[1])
        self.assertTrue(claimed["question_required"])
        persisted = answers.read_text(encoding="utf-8")
        self.assertIn("question_claimed_at_unix", persisted)
        self.assertNotIn("session", persisted.lower())
        self.assertNotIn("codex", persisted.lower())

    def test_three_host_skill_projection_sources_keep_the_same_onboarding_route(self) -> None:
        paths = (
            ROOT / "harness/skills/user-setup/SKILL.md",
            ROOT / "harness/plugins/aigent-hive/skills/user-setup/SKILL.md",
            ROOT / "harness/template/.agents/skills/user-setup/SKILL.md",
            ROOT / "harness/template/.claude/skills/user-setup/SKILL.md",
        )
        source = paths[0].read_bytes()
        for path in paths[1:]:
            with self.subTest(path=path):
                self.assertEqual(path.read_bytes(), source)
        text = source.decode("utf-8")
        for required in (
            "hive setup feature claim --id vector-search",
            "question_required",
            "claim expires",
            "setup_request_digest",
            "never store a host session identifier",
        ):
            with self.subTest(required=required):
                self.assertIn(required, text)
