"""Current product Skill identity and projection contracts for v0.9."""

from __future__ import annotations

import hashlib
import unittest
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[2]
SKILL_ROOT = ROOT / "harness/skills"
PLUGIN_ROOT = ROOT / "harness/plugins/aigent-hive/skills"
TEMPLATE_ROOT = ROOT / "harness/template/.agents/skills"
CLAUDE_TEMPLATE_ROOT = ROOT / "harness/template/.claude/skills"
SOURCE_SKILL_ROOT = ROOT / ".agents/skills"

EXPECTED_SKILLS = frozenset(
    {
        "quick-answer",
        "project-setup",
        "code-polish",
        "custom-subagent-create",
        "iterative-execution",
        "ralph-loop",
        "knowledge-import",
        "knowledge-maintain",
        "knowledge-capture",
        "prompt-refine",
        "research-best-practices",
        "knowledge-recall",
        "usage-guard",
        "ship",
        "amend-directive",
        "user-setup",
        "run-handoff",
        "project-transition",
        "run-resume",
        "run-checkpoint",
        "knowledge-promote",
        "multi-goal",
        "product-update",
        "project-refresh",
        "package-review",
        "team-execution",
    }
)


def skill_names(root: Path) -> set[str]:
    return {path.parent.name for path in root.glob("*/SKILL.md")}


class V09HiveSkillProjectionTests(unittest.TestCase):
    def test_current_product_inventory_is_exact_and_source_has_no_skill_body(self) -> None:
        self.assertEqual(skill_names(SKILL_ROOT), EXPECTED_SKILLS)
        self.assertEqual(skill_names(PLUGIN_ROOT), EXPECTED_SKILLS)
        self.assertEqual(skill_names(TEMPLATE_ROOT), EXPECTED_SKILLS)
        self.assertEqual(skill_names(CLAUDE_TEMPLATE_ROOT), EXPECTED_SKILLS)
        self.assertFalse(SOURCE_SKILL_ROOT.exists() and list(SOURCE_SKILL_ROOT.rglob("SKILL.md")))

    def test_each_current_product_skill_is_byte_identical_in_all_projections(self) -> None:
        for name in EXPECTED_SKILLS:
            canonical = (SKILL_ROOT / name / "SKILL.md").read_bytes()
            self.assertEqual(
                (PLUGIN_ROOT / name / "SKILL.md").read_bytes(), canonical, name
            )
            self.assertEqual(
                (TEMPLATE_ROOT / name / "SKILL.md").read_bytes(), canonical, name
            )
            self.assertEqual(
                (CLAUDE_TEMPLATE_ROOT / name / "SKILL.md").read_bytes(), canonical, name
            )

    def test_all_hive_skill_descriptions_start_with_the_canonical_id(self) -> None:
        catalog = yaml.safe_load((SKILL_ROOT / "catalog.yml").read_text(encoding="utf-8"))
        descriptions = {entry["name"]: entry["description"] for entry in catalog["skills"]}
        self.assertEqual(set(descriptions), EXPECTED_SKILLS)
        for name in EXPECTED_SKILLS:
            expected = f"({name})"
            with self.subTest(name=name):
                self.assertTrue(descriptions[name].startswith(expected))
                text = (SKILL_ROOT / name / "SKILL.md").read_text(encoding="utf-8")
                description = next(
                    line.removeprefix("description: ")
                    for line in text.splitlines()
                    if line.startswith("description: ")
                )
                self.assertTrue(description.strip('"').startswith(expected))
                metadata = yaml.safe_load(
                    (SKILL_ROOT / name / "agents" / "openai.yaml").read_text(
                        encoding="utf-8"
                    )
                )
                self.assertTrue(
                    metadata["interface"]["short_description"].startswith(expected)
                )

    def test_catalog_schema_and_ledger_emit_current_names_only(self) -> None:
        catalog = yaml.safe_load((SKILL_ROOT / "catalog.yml").read_text(encoding="utf-8"))
        self.assertEqual({item["name"] for item in catalog["skills"]}, EXPECTED_SKILLS)

        schema = yaml.safe_load(
            (ROOT / "schemas/user-setup.schema.json").read_text(encoding="utf-8")
        )
        self.assertEqual(
            set(schema["$defs"]["skill_name"]["enum"]), EXPECTED_SKILLS
        )

        ledger = yaml.safe_load((SKILL_ROOT / "retired-names.yml").read_text(encoding="utf-8"))
        retired = ledger["retired_names"]
        self.assertTrue(set(retired).isdisjoint(EXPECTED_SKILLS))
        self.assertTrue(set(retired.values()).issubset(EXPECTED_SKILLS))
        self.assertTrue(set(retired).isdisjoint(skill_names(SKILL_ROOT)))

    def test_short_names_keep_the_aigent_hive_provider_namespace(self) -> None:
        for name in EXPECTED_SKILLS:
            text = (SKILL_ROOT / name / "SKILL.md").read_text(encoding="utf-8")
            self.assertIn(f"name: {name}", text)
            self.assertNotIn("name: hive-", text)
        catalog = (ROOT / "docs/skills.md").read_text(encoding="utf-8")
        self.assertIn("$aigent-hive:<Skill 이름>", catalog)

    def test_user_setup_keeps_the_bilingual_response_language_contract(self) -> None:
        text = (SKILL_ROOT / "user-setup/SKILL.md").read_text(encoding="utf-8")
        for expected in (
            "## Response language contract",
            "ASD-STE100 Simplified Technical English",
            "Translate meaning rather than English word order.",
            "mixed Korean-English compounds",
            "benign한 source claim ID",
            "원본 지식 항목 식별자",
        ):
            self.assertIn(expected, text)

    def test_iterative_execution_keeps_role_parity_on_one_strict_judge_path(self) -> None:
        text = (SKILL_ROOT / "iterative-execution" / "SKILL.md").read_text(
            encoding="utf-8"
        )
        for route in (
            "planning after `$aigent-hive:ralph-loop`",
            "independent review or\n  QA after `$aigent-hive:package-review`",
            "evidence-backed research after\n  `$aigent-hive:research-best-practices`",
            "performance validation with declared measurements",
            "Terminal acceptance always requires the reserved independent Judge.",
        ):
            self.assertIn(route, text)

    def test_rename_ledger_is_stable_and_complete_for_the_last_public_inventory(self) -> None:
        ledger = yaml.safe_load((SKILL_ROOT / "retired-names.yml").read_text(encoding="utf-8"))
        retired = ledger["retired_names"]
        for retired_name in (
            "auto-setup-project",
            "configure",
            "manage-usage",
            "manage-wiki",
            "refine-prompt",
            "search-knowledge",
            "setup-project",
            "hive-usage-guard",
            "setup-harness",
        ):
            self.assertIn(retired_name, retired)
        self.assertEqual(
            hashlib.sha256((SKILL_ROOT / "retired-names.yml").read_bytes()).hexdigest(),
            hashlib.sha256((SKILL_ROOT / "retired-names.yml").read_bytes()).hexdigest(),
        )


if __name__ == "__main__":
    unittest.main()
