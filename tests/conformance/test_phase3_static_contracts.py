"""Static contracts for the current product-only v0.9 Skill suite."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[2]
SKILLS = ROOT / "harness/skills"
PLUGIN_SKILLS = ROOT / "harness/plugins/aigent-hive/skills"
TEMPLATE_SKILLS = ROOT / "harness/template/.agents/skills"

CURRENT = {
    "quick-answer", "project-setup", "code-polish", "ralph-loop", "knowledge-import",
    "knowledge-maintain", "knowledge-capture", "prompt-refine", "research-best-practices",
    "knowledge-recall", "usage-guard", "ship", "amend-directive", "user-setup",
    "run-handoff", "project-transition", "run-resume", "run-checkpoint",
    "knowledge-promote", "product-update", "project-refresh", "package-review",
}


def skill_paths(root: Path) -> set[str]:
    return {entry.parent.name for entry in root.glob("*/SKILL.md")}


class Phase3SchemaContract(unittest.TestCase):
    def test_current_catalog_is_machine_readable_and_exact(self) -> None:
        catalog = yaml.safe_load((SKILLS / "catalog.yml").read_text(encoding="utf-8"))
        self.assertEqual(catalog["schema_version"], 1)
        self.assertEqual({entry["name"] for entry in catalog["skills"]}, CURRENT)
        self.assertTrue(all(entry["availability"] == "implemented" for entry in catalog["skills"]))

    def test_setup_schema_has_only_current_skill_choices(self) -> None:
        schema = json.loads((ROOT / "schemas/user-setup.schema.json").read_text(encoding="utf-8"))
        self.assertEqual(set(schema["$defs"]["skill_name"]["enum"]), CURRENT)
        self.assertEqual(len(schema["$defs"]["skill_name"]["enum"]), len(CURRENT))


class Phase3SkillSourceContract(unittest.TestCase):
    def test_product_skill_projections_are_current_and_exact(self) -> None:
        self.assertEqual(skill_paths(SKILLS), CURRENT)
        self.assertEqual(skill_paths(PLUGIN_SKILLS), CURRENT)
        self.assertEqual(skill_paths(TEMPLATE_SKILLS), CURRENT)
        for name in CURRENT:
            canonical = (SKILLS / name / "SKILL.md").read_bytes()
            self.assertEqual((PLUGIN_SKILLS / name / "SKILL.md").read_bytes(), canonical)
            self.assertEqual((TEMPLATE_SKILLS / name / "SKILL.md").read_bytes(), canonical)

    def test_product_skill_text_does_not_contain_a_renamed_skill_as_prose(self) -> None:
        """The global Skill ID must not leak into ordinary setup instructions."""
        forbidden = ("reuser-setup", "user-setupd", "user-setuping")
        for skill_path in SKILLS.glob("*/SKILL.md"):
            text = skill_path.read_text(encoding="utf-8").lower()
            for value in forbidden:
                self.assertNotIn(value, text, skill_path)

    def test_source_has_directives_not_a_second_skill_inventory(self) -> None:
        source_skills = ROOT / ".agents/skills"
        self.assertFalse(source_skills.exists() and list(source_skills.rglob("SKILL.md")))
        agents = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
        architecture = (ROOT / ".agents/directives/02-architecture.md").read_text(encoding="utf-8")
        self.assertIn("no separate tracked Skill inventory", agents)
        self.assertIn("Source-root, usage-gate, and mutation boundaries belong in repository directives", architecture)

    def test_source_routes_prompt_and_wiki_work_to_current_product_contracts(self) -> None:
        behavior = (ROOT / ".agents/directives/01-behavior.md").read_text(encoding="utf-8")
        agents = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
        self.assertIn("automatically load `aigent-hive:prompt-refine`", behavior)
        self.assertIn("hive source-wiki query --target <source-root>", behavior)
        self.assertIn("scripts/source-usage-guard.py gate --json", agents)
        self.assertNotIn("Run the source `hive-usage-guard`", agents)

    def test_global_setup_contract_uses_describe_progress_and_conditional_integrations(self) -> None:
        skill = (SKILLS / "user-setup/SKILL.md").read_text(encoding="utf-8")
        for required in (
            "hive setup --scope user --describe --output json",
            "Get-Command hive",
            "where.exe hive",
            "npm prefix -g",
            "hive setup --progress save --scope user",
            "no default usage threshold",
            "Continue from where I left off",
            "Discord usage notification",
            "CodexBar fallback",
        ):
            self.assertIn(required, skill)
        self.assertIn("Do not translate `Skill` as `기술`", skill)
        resolver = SKILLS / "user-setup/scripts/resolve-hive.ps1"
        self.assertTrue(resolver.is_file())
        resolver_text = resolver.read_text(encoding="utf-8")
        for required in ("Get-Command hive", "where.exe hive", "npm prefix -g", "--version"):
            self.assertIn(required, resolver_text)

        copier = (ROOT / "copier.yml").read_text(encoding="utf-8")
        threshold = copier.split("usage_stop_remaining_percent:", 1)[1].split(
            "\nelevated_judge_quorum:", 1
        )[0]
        self.assertNotIn("default:", threshold)

    def test_discord_visual_guide_is_shipped_with_the_user_template(self) -> None:
        guide = ROOT / "harness/template/.hive/guides/discord-usage-notifications.html"
        self.assertTrue(guide.is_file())
        text = guide.read_text(encoding="utf-8")
        self.assertIn("HIVE_DISCORD_WEBHOOK_URL", text)
        self.assertIn("raw prompts", text)

    def test_rename_ledger_keeps_old_ids_out_of_live_products(self) -> None:
        ledger = yaml.safe_load((SKILLS / "retired-names.yml").read_text(encoding="utf-8"))
        retired = set(ledger["retired_names"])
        self.assertTrue(retired.isdisjoint(CURRENT))
        self.assertTrue(set(ledger["retired_names"].values()).issubset(CURRENT))
        self.assertTrue(retired.isdisjoint(skill_paths(SKILLS)))

    def test_new_universal_skill_boundaries_are_present(self) -> None:
        ship = (SKILLS / "ship/SKILL.md").read_text(encoding="utf-8")
        amend = (SKILLS / "amend-directive/SKILL.md").read_text(encoding="utf-8")
        self.assertIn("explicit", ship.lower())
        self.assertIn("foreign", amend.lower())
        self.assertIn("signed", amend.lower())


if __name__ == "__main__":
    unittest.main()
