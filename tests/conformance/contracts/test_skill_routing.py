#!/usr/bin/env python3
"""Deterministic precedence tests over normalized routing facts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.support.harness import (
    ACTION_RESULT_SCHEMA,
    Phase1CliTestCase,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
FIXTURE_ROOT = REPOSITORY_ROOT / "tests/fixtures/skills/routes"
REQUEST_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/routing-request.schema.json").read_text(
        encoding="utf-8"
    )
)
DECISION_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/routing-decision.schema.json").read_text(
        encoding="utf-8"
    )
)


class Phase3RoutingContract(Phase1CliTestCase):
    def invoke_route(
        self,
        name: str,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        process = subprocess.run(
            [
                str(self.hive_binary),
                "route",
                "--request",
                str(FIXTURE_ROOT / name),
                "--output",
                "json",
            ],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"route stdout must be one JSON object: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"])
        data = result.get("data")
        if data is not None:
            self.assertIsInstance(data, dict)
            Draft202012Validator(
                DECISION_SCHEMA,
                format_checker=FormatChecker(),
            ).validate(data)
        return process, result

    def decision(self, result: dict[str, object]) -> dict[str, object]:
        data = result["data"]
        self.assertIsInstance(data, dict)
        return data

    def test_route_requests_contain_normalized_facts_not_raw_prompts(
        self,
    ) -> None:
        self.assertNotIn("prompt", REQUEST_SCHEMA["properties"])
        for path in sorted(FIXTURE_ROOT.glob("*.json")):
            with self.subTest(path=path):
                request = json.loads(path.read_text(encoding="utf-8"))
                Draft202012Validator(
                    REQUEST_SCHEMA,
                    format_checker=FormatChecker(),
                ).validate(request)
                self.assertNotIn("prompt", request)
                self.assertNotIn("text", request)
                self.assertNotIn("intent", request)

    def test_simple_question_precedes_hive_candidate(self) -> None:
        process, result = self.invoke_route("simple-question.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(
            self.decision(result),
            {
                "schema_version": 1,
                "route": "simple-question",
                "logical_action": "AnswerSimpleQuestion",
                "selected_skill": "quick-answer",
                "provided_by": "hive",
                "mode": None,
                "load_skill_bodies": ["quick-answer"],
                "next_action": None,
            },
        )

    def test_plain_answer_precedes_automatic_prompt_refine(self) -> None:
        process, result = self.invoke_route("plain-answer.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "direct")
        self.assertEqual(decision["logical_action"], "AnswerSimpleQuestion")
        self.assertIsNone(decision["selected_skill"])
        self.assertEqual(decision["load_skill_bodies"], [])

    def test_ambiguous_work_automatically_selects_refine_only(self) -> None:
        process, result = self.invoke_route("ambiguous-work.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "hive-skill")
        self.assertEqual(decision["logical_action"], "RefinePrompt")
        self.assertFalse(decision.get("refine_suggestion", False))
        self.assertEqual(decision["selected_skill"], "prompt-refine")
        self.assertEqual(decision["load_skill_bodies"], ["prompt-refine"])
        self.assertEqual(decision["mode"], "refine-only")

    def test_prompt_refine_defaults_to_refine_only(self) -> None:
        process, result = self.invoke_route("prompt-refine.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "hive-skill")
        self.assertEqual(decision["logical_action"], "RefinePrompt")
        self.assertEqual(decision["selected_skill"], "prompt-refine")
        self.assertEqual(decision["mode"], "refine-only")
        self.assertEqual(
            decision["load_skill_bodies"],
            ["prompt-refine"],
        )

    def test_explicitly_selected_compatible_omx_skill_precedes_hive_candidate(self) -> None:
        process, result = self.invoke_route("omx-analyze.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "external-skill")
        self.assertEqual(decision["selected_skill"], "analyze")
        self.assertEqual(decision["provided_by"], "omx")
        self.assertEqual(decision["load_skill_bodies"], ["analyze"])

    def test_compatible_but_unselected_omx_uses_host_native_route(self) -> None:
        process, result = self.invoke_route("omx-analyze-unselected.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "host-native")
        self.assertIsNone(decision["selected_skill"])
        self.assertEqual(decision["provided_by"], "host-native")
        self.assertEqual(decision["load_skill_bodies"], [])

    def test_explicit_skill_precedes_simple_and_external_candidates(self) -> None:
        process, result = self.invoke_route("explicit-hive-skill.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "hive-skill")
        self.assertEqual(decision["logical_action"], "QueryKnowledge")
        self.assertEqual(
            decision["selected_skill"],
            "knowledge-recall",
        )
        self.assertEqual(
            decision["load_skill_bodies"],
            ["knowledge-recall"],
        )

    def test_project_dependent_simple_question_is_not_auto_transitioned(
        self,
    ) -> None:
        process, result = self.invoke_route("project-dependent-simple.json")
        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["changed_paths"], [])
        decision = self.decision(result)
        self.assertEqual(decision["route"], "blocked")
        self.assertEqual(decision["next_action"], "RunWork")
        self.assertEqual(decision["load_skill_bodies"], [])

    def test_implicit_refine_and_run_is_blocked(self) -> None:
        process, result = self.invoke_route("implicit-refine-and-run.json")
        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["changed_paths"], [])
        decision = self.decision(result)
        self.assertEqual(decision["route"], "blocked")
        self.assertEqual(decision["logical_action"], "RefinePrompt")
        self.assertEqual(decision["load_skill_bodies"], [])

    def test_explicit_refine_and_run_is_selected(self) -> None:
        process, result = self.invoke_route("explicit-refine-and-run.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["logical_action"], "RefinePrompt")
        self.assertEqual(decision["selected_skill"], "prompt-refine")
        self.assertEqual(decision["mode"], "refine-and-run")

    def test_host_native_is_last_precedence_fallback(self) -> None:
        process, result = self.invoke_route("host-native.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "host-native")
        self.assertEqual(decision["logical_action"], "RunWork")
        self.assertIsNone(decision["selected_skill"])
        self.assertEqual(decision["load_skill_bodies"], [])

    def test_complex_work_routes_to_verified_workflow_with_reason_codes(self) -> None:
        process, result = self.invoke_route("complex-verified-workflow.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "hive-skill")
        self.assertEqual(decision["selected_skill"], "verified-workflow")
        self.assertEqual(decision["workflow_route"], "verified-workflow")
        self.assertEqual(
            decision["workflow_reason_codes"],
            ["dependency-graph", "independent-verifier"],
        )

    def test_complex_work_blocks_when_verified_workflow_is_inactive(self) -> None:
        process, result = self.invoke_route("complex-verified-workflow-inactive.json")
        self.assertEqual(process.returncode, 3, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "blocked")
        self.assertEqual(decision["workflow_route"], "required-but-unsupported")

    def test_inactive_automatic_hive_candidate_is_blocked(self) -> None:
        process, result = self.invoke_route("inactive-hive-candidate.json")
        self.assertEqual(process.returncode, 3, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "blocked")
        self.assertIsNone(decision["selected_skill"])
        self.assertEqual(decision["load_skill_bodies"], [])

    def test_explicit_inactive_unnamespaced_hive_skill_is_blocked(
        self,
    ) -> None:
        process, result = self.invoke_route("explicit-inactive-hive-skill.json")
        self.assertEqual(process.returncode, 3, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "blocked")
        self.assertIsNone(decision["selected_skill"])
        self.assertEqual(decision["load_skill_bodies"], [])

    def test_unapproved_optional_hive_skill_is_verification_failure(
        self,
    ) -> None:
        process, result = self.invoke_route(
            "unapproved-optional-hive-skill.json"
        )
        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(result["code"], "hive.routing-proof-invalid")
        self.assertEqual(result["changed_paths"], [])
        self.assertNotIn("data", result)

    def test_forged_digest_or_consent_is_verification_failure(self) -> None:
        for fixture in (
            "forged-builtin-digest.json",
            "forged-optional-consent.json",
        ):
            with self.subTest(fixture=fixture):
                process, result = self.invoke_route(fixture)
                self.assertEqual(process.returncode, 5, process.stderr)
                self.assertEqual(result["status"], "verification-failed")
                self.assertEqual(result["code"], "hive.routing-proof-invalid")
                self.assertEqual(result["changed_paths"], [])
                self.assertNotIn("data", result)

    def test_approved_optional_hive_skill_loads_exactly_one_body(self) -> None:
        process, result = self.invoke_route("approved-optional-hive-skill.json")
        self.assertEqual(process.returncode, 0, process.stderr)
        decision = self.decision(result)
        self.assertEqual(decision["route"], "hive-skill")
        self.assertEqual(decision["selected_skill"], "local-inspect")
        self.assertEqual(decision["provided_by"], "hive")
        self.assertEqual(decision["load_skill_bodies"], ["local-inspect"])

    def test_every_route_loads_at_most_one_skill_body(self) -> None:
        for path in sorted(FIXTURE_ROOT.glob("*.json")):
            with self.subTest(path=path):
                _, result = self.invoke_route(path.name)
                if "data" not in result:
                    self.assertEqual(result["status"], "verification-failed")
                    continue
                self.assertLessEqual(
                    len(self.decision(result)["load_skill_bodies"]),
                    1,
                )


if __name__ == "__main__":
    import unittest

    unittest.main()
