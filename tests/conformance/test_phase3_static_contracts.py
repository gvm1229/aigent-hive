#!/usr/bin/env python3
"""Static shipped-contract gates for Phase 3 portable Skills."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SKILL_ROOT = REPOSITORY_ROOT / "harness/skills"
CATALOG_PATH = SKILL_ROOT / "catalog.yml"
CAPABILITY_SCHEMA_PATH = REPOSITORY_ROOT / "schemas/capability-matrix.schema.json"
PROMPT_INPUT_SCHEMA_PATH = (
    REPOSITORY_ROOT / "schemas/prompt-refinement-input.schema.json"
)
PROMPT_RESULT_SCHEMA_PATH = (
    REPOSITORY_ROOT / "schemas/prompt-refinement-result.schema.json"
)
SKILL_CATALOG_SCHEMA_PATH = (
    REPOSITORY_ROOT / "schemas/skill-catalog.schema.json"
)
ROUTING_DECISION_SCHEMA_PATH = (
    REPOSITORY_ROOT / "schemas/routing-decision.schema.json"
)
ROUTING_REQUEST_SCHEMA_PATH = (
    REPOSITORY_ROOT / "schemas/routing-request.schema.json"
)

BUILTIN_SKILLS = {
    "setup-harness",
    "hive-simple-question",
    "hive-prompt-refine",
    "hive-knowledge-capture",
    "hive-knowledge-query",
    "hive-knowledge-maintenance",
    "hive-run-checkpoint",
    "hive-run-resume",
    "hive-role-handoff",
    "hive-judge-package",
    "hive-update",
    "hive-migrate",
}
IMPLEMENTED_SKILLS = {
    "setup-harness",
    "hive-simple-question",
    "hive-prompt-refine",
    "hive-knowledge-capture",
    "hive-knowledge-query",
    "hive-knowledge-maintenance",
    "hive-run-checkpoint",
    "hive-run-resume",
    "hive-role-handoff",
    "hive-judge-package",
}
CATALOG_ONLY_SKILLS = BUILTIN_SKILLS - IMPLEMENTED_SKILLS


def read_yaml(path: Path) -> dict[str, object]:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected YAML object: {path}")
    return value


def skill_frontmatter(path: Path) -> tuple[dict[str, object], str]:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        raise AssertionError(f"missing YAML frontmatter: {path}")
    frontmatter, separator, body = text[4:].partition("\n---\n")
    if not separator:
        raise AssertionError(f"unterminated YAML frontmatter: {path}")
    value = yaml.safe_load(frontmatter)
    if not isinstance(value, dict):
        raise AssertionError(f"frontmatter must be an object: {path}")
    return value, body


class Phase3SkillSourceContract(unittest.TestCase):
    def test_catalog_contains_every_phase_three_builtin_name(self) -> None:
        catalog = read_yaml(CATALOG_PATH)
        skills = catalog.get("skills")
        self.assertIsInstance(skills, list)
        names = {
            entry["name"]
            for entry in skills
            if isinstance(entry, dict) and isinstance(entry.get("name"), str)
        }
        self.assertEqual(names, BUILTIN_SKILLS)

    def test_catalog_contains_no_orchestration_clone(self) -> None:
        text = CATALOG_PATH.read_text(encoding="utf-8").casefold()
        forbidden = ("hive-plan", "hive-ralph", "hive-team", "hive-swarm")
        for name in forbidden:
            with self.subTest(name=name):
                self.assertNotIn(name, text)

    def test_catalog_entries_declare_routing_and_side_effect_metadata(self) -> None:
        catalog = read_yaml(CATALOG_PATH)
        skills = catalog.get("skills")
        self.assertIsInstance(skills, list)
        required = {
            "name",
            "description",
            "provided_by",
            "superseded_by_external",
            "invocation_intents",
            "side_effect_class",
            "capabilities",
            "availability",
        }
        for entry in skills:
            with self.subTest(entry=entry):
                self.assertIsInstance(entry, dict)
                self.assertTrue(required.issubset(entry))
                self.assertIsInstance(entry["invocation_intents"], list)

    def test_implemented_catalog_entries_have_exact_skill_source(self) -> None:
        for name in sorted(IMPLEMENTED_SKILLS):
            with self.subTest(name=name):
                path = SKILL_ROOT / name / "SKILL.md"
                frontmatter, _ = skill_frontmatter(path)
                self.assertEqual(set(frontmatter), {"name", "description"})
                self.assertEqual(frontmatter["name"], name)
                self.assertIsInstance(frontmatter["description"], str)
                self.assertTrue(frontmatter["description"].strip())

    def test_catalog_only_entries_have_no_discoverable_skill_body(self) -> None:
        for name in sorted(CATALOG_ONLY_SKILLS):
            with self.subTest(name=name):
                self.assertFalse((SKILL_ROOT / name / "SKILL.md").exists())

    def test_phase_four_data_contract_skills_use_only_exact_cli_surfaces(
        self,
    ) -> None:
        expected = {
            "hive-run-checkpoint": "hive run checkpoint",
            "hive-run-resume": "hive run resume",
            "hive-role-handoff": "hive role handoff",
        }
        for name, command in expected.items():
            with self.subTest(name=name):
                source = SKILL_ROOT / name / "SKILL.md"
                _, body = skill_frontmatter(source)
                normalized = body.casefold()
                self.assertIn(command, normalized)
                self.assertIn("--output json", normalized)
                self.assertIn("never", normalized)
                self.assertNotIn("provider api key", normalized)
                for forbidden_command in (
                    "\n   omx ",
                    "\n   omc ",
                    "\n   hive plan ",
                    "\n   hive team ",
                ):
                    self.assertNotIn(forbidden_command, normalized)

    def test_phase_four_template_skill_bytes_match_canonical_sources(self) -> None:
        template_root = (
            REPOSITORY_ROOT
            / "harness/template/{{ '.claude' if primary_host == 'claude' else '.agents' }}/skills"
        )
        for name in (
            "hive-run-checkpoint",
            "hive-run-resume",
            "hive-role-handoff",
        ):
            with self.subTest(name=name):
                self.assertEqual(
                    (template_root / name / "SKILL.md").read_bytes(),
                    (SKILL_ROOT / name / "SKILL.md").read_bytes(),
                )

    def test_forbidden_prompt_aliases_are_absent_from_shipped_sources(self) -> None:
        shipped_roots = (
            REPOSITORY_ROOT / "harness",
            REPOSITORY_ROOT / "schemas",
        )
        forbidden = ("hive-prompt-perfect", "prompt perfect")
        for root in shipped_roots:
            for path in root.rglob("*"):
                if not path.is_file():
                    continue
                text = path.read_text(encoding="utf-8", errors="ignore").casefold()
                for alias in forbidden:
                    with self.subTest(path=path, alias=alias):
                        self.assertNotIn(alias, text)

    def test_simple_question_source_forbids_project_capabilities(self) -> None:
        _, body = skill_frontmatter(
            SKILL_ROOT / "hive-simple-question/SKILL.md"
        )
        normalized = body.casefold()
        required_boundaries = (
            "project memory",
            "wiki",
            "write",
            "subagent",
            "orchestration",
            "run",
            "transition",
        )
        for boundary in required_boundaries:
            with self.subTest(boundary=boundary):
                self.assertIn(boundary, normalized)

    def test_prompt_refine_source_defaults_to_refine_only(self) -> None:
        frontmatter, body = skill_frontmatter(
            SKILL_ROOT / "hive-prompt-refine/SKILL.md"
        )
        description = str(frontmatter["description"]).casefold()
        normalized = body.casefold()
        self.assertIn("prompt", description)
        self.assertIn("refine-only", normalized)
        self.assertIn("default", normalized)

    def test_prompt_refine_source_requires_explicit_refine_and_run_intent(
        self,
    ) -> None:
        _, body = skill_frontmatter(
            SKILL_ROOT / "hive-prompt-refine/SKILL.md"
        )
        normalized = body.casefold()
        self.assertIn("refine-and-run", normalized)
        self.assertIn("explicit", normalized)

    def test_prompt_refine_source_forbids_hidden_rewrite(self) -> None:
        _, body = skill_frontmatter(
            SKILL_ROOT / "hive-prompt-refine/SKILL.md"
        )
        normalized = body.casefold()
        self.assertIn("ordinary", normalized)
        self.assertIn("rewrite", normalized)
        self.assertIn("do not", normalized)

    def test_prompt_refine_source_declares_meaning_preservation(self) -> None:
        _, body = skill_frontmatter(
            SKILL_ROOT / "hive-prompt-refine/SKILL.md"
        )
        normalized = body.casefold()
        for field in ("must", "must-not", "scope", "output", "authority"):
            with self.subTest(field=field):
                self.assertIn(field, normalized)


class Phase3SchemaContract(unittest.TestCase):
    def validate_capability_fixture(self, path: Path) -> None:
        schema = json.loads(
            CAPABILITY_SCHEMA_PATH.read_text(encoding="utf-8")
        )
        instance = json.loads(path.read_text(encoding="utf-8"))
        Draft202012Validator(
            schema,
            format_checker=FormatChecker(),
        ).validate(instance)

    def test_prompt_refinement_input_schema_has_required_modes(self) -> None:
        schema = json.loads(
            PROMPT_INPUT_SCHEMA_PATH.read_text(encoding="utf-8")
        )
        mode = schema["properties"]["mode"]
        self.assertEqual(set(mode["enum"]), {"refine-only", "refine-and-run"})

    def test_prompt_refinement_result_schema_preserves_original_and_refined_text(
        self,
    ) -> None:
        schema = json.loads(
            PROMPT_RESULT_SCHEMA_PATH.read_text(encoding="utf-8")
        )
        required = set(schema["required"])
        self.assertTrue({"original_prompt", "refined_prompt"}.issubset(required))

    def test_skill_catalog_is_machine_schema_valid(self) -> None:
        schema = json.loads(
            SKILL_CATALOG_SCHEMA_PATH.read_text(encoding="utf-8")
        )
        Draft202012Validator(
            schema,
            format_checker=FormatChecker(),
        ).validate(read_yaml(CATALOG_PATH))

    def test_routing_decision_schema_limits_skill_body_load_to_one(
        self,
    ) -> None:
        schema = json.loads(
            ROUTING_DECISION_SCHEMA_PATH.read_text(encoding="utf-8")
        )
        self.assertEqual(
            schema["properties"]["load_skill_bodies"]["maxItems"],
            1,
        )

    def test_routing_request_requires_digest_bound_active_skill_proofs(
        self,
    ) -> None:
        schema = json.loads(
            ROUTING_REQUEST_SCHEMA_PATH.read_text(encoding="utf-8")
        )
        self.assertIn("active_hive_skills", schema["required"])
        proof = schema["$defs"]["activeHiveSkillProof"]
        self.assertTrue(
            {
                "name",
                "source_type",
                "content_digest",
                "side_effect_class",
                "capabilities",
                "consent_digest",
                "consent",
            }.issubset(proof["required"])
        )
        Draft202012Validator.check_schema(schema)

    def test_capability_matrix_declares_phase_three_capabilities(self) -> None:
        schema = json.loads(
            CAPABILITY_SCHEMA_PATH.read_text(encoding="utf-8")
        )
        capabilities = schema["properties"]["capabilities"]
        self.assertTrue(
            {"automatic-skill-routing", "prompt-refine"}.issubset(
                capabilities["properties"]
            )
        )
        self.assertIn("hook_events", schema["properties"])
        self.assertNotIn("automatic-skill-routing", capabilities["required"])
        self.assertNotIn("prompt-refine", capabilities["required"])
        self.assertNotIn("hook_events", schema["required"])

    def test_hook_event_capabilities_include_explicit_support_evidence(
        self,
    ) -> None:
        schema = json.loads(
            CAPABILITY_SCHEMA_PATH.read_text(encoding="utf-8")
        )
        hook_events = schema["properties"]["hook_events"]
        properties = hook_events["properties"]
        self.assertEqual(
            set(properties),
            {
                "UserPromptSubmit",
                "PreToolUse",
                "PostToolUse",
                "PreCompact",
                "Stop",
            },
        )
        claim = schema["$defs"]["supportClaim"]
        self.assertEqual(set(claim["required"]), {"support", "evidence"})

    def test_enriched_phase_three_capability_fixture_is_schema_valid(
        self,
    ) -> None:
        self.validate_capability_fixture(
            REPOSITORY_ROOT
            / "tests/fixtures/phase3/capabilities-codex-enriched.json"
        )

    def test_legacy_phase_one_capability_fixture_remains_schema_valid(
        self,
    ) -> None:
        self.validate_capability_fixture(
            REPOSITORY_ROOT
            / "tests/fixtures/phase1/capabilities-codex-omx.json"
        )


if __name__ == "__main__":
    unittest.main()
