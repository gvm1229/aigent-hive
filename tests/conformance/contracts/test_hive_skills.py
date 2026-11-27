"""Current product Skill identity and projection contracts for v0.9."""

from __future__ import annotations

import hashlib
import unittest
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[3]
SKILL_ROOT = ROOT / "harness/skills"
PLUGIN_ROOT = ROOT / "harness/plugins/aigent-hive/skills"
TEMPLATE_ROOT = ROOT / "harness/template/.agents/skills"
CLAUDE_TEMPLATE_ROOT = ROOT / "harness/template/.claude/skills"
SOURCE_SKILL_ROOT = ROOT / ".agents/skills"

EXPECTED_SKILLS = frozenset(
    {
        "athena",
        "artemis",
        "hephaistos",
        "apollon",
        "eileithyia",
        "poseidon",
        "demeter",
        "aither",
        "hades",
        "quick-answer",
        "project-setup",
        "ai-slop-cleaner",
        "humanize-kor",
        "custom-subagent-create",
        "autonomous-workflow",
        "knowledge-manage",
        "knowledge-capture",
        "prompt-refine",
        "research-best-practices",
        "knowledge-recall",
        "usage-guard",
        "ship",
        "amend-directive",
        "user-setup",
        "run",
        "project-transition",
        "multi-goal",
        "product-update",
        "project-refresh",
        "judge",
        "team-execution",
    }
)


def skill_names(root: Path) -> set[str]:
    return {path.parent.name for path in root.glob("*/SKILL.md")}


class V09HiveSkillProjectionTests(unittest.TestCase):
    def test_current_product_inventory_is_exact_with_source_only_exceptions(self) -> None:
        self.assertEqual(skill_names(SKILL_ROOT), EXPECTED_SKILLS)
        self.assertEqual(skill_names(PLUGIN_ROOT), EXPECTED_SKILLS)
        self.assertEqual(skill_names(TEMPLATE_ROOT), EXPECTED_SKILLS)
        self.assertEqual(skill_names(CLAUDE_TEMPLATE_ROOT), EXPECTED_SKILLS)
        self.assertEqual(
            skill_names(SOURCE_SKILL_ROOT),
            {"autonomous-workflow", "draft-devlog", "update-summary"},
        )
        for name in ("draft-devlog", "update-summary"):
            source_skill = (SOURCE_SKILL_ROOT / name / "SKILL.md").read_text(
                encoding="utf-8"
            )
            self.assertIn("source-project-only", source_skill)
            self.assertTrue(
                "never enters `harness/`" in source_skill
                or "Do not add it to `harness/`" in source_skill
            )

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
        text = (SKILL_ROOT / "user-setup/references/language.md").read_text(encoding="utf-8")
        for expected in (
            "## Response language contract",
            "ASD-STE100 Simplified Technical English",
            "Translate meaning rather than English word order.",
            "mixed Korean-English compounds",
            "benign한 source claim ID",
            "원본 지식 항목 식별자",
        ):
            self.assertIn(expected, text)

    def test_usage_guard_core_stays_short_and_names_its_detail_sections(self) -> None:
        # OV160-008: whitespace word count of the whole SKILL.md, as `wc -w` measures it.
        core = (SKILL_ROOT / "usage-guard/SKILL.md").read_text(encoding="utf-8")
        self.assertLessEqual(len(core.split()), 500)
        details = (SKILL_ROOT / "usage-guard/references/control.md").read_text(encoding="utf-8")
        headings = {line[3:] for line in details.splitlines() if line.startswith("## ")}
        for section in (
            "Target and policy",
            "Automatic-dispatch preflight",
            "Setup and manual inspection",
            "Boundaries detail",
            "Windows midway monitor",
        ):
            self.assertIn(f"*{section}*", core)
            self.assertIn(section, headings)
        for rule in (
            "Never guess a percentage",
            "Never install a fallback without explicit current-action acceptance",
            "never raw prompts or credentials",
            "cannot clear an exhaustion halt",
        ):
            self.assertIn(rule, " ".join(core.split()))

    def test_setup_routers_use_one_cli_state_and_no_duplicate_questionnaire(self) -> None:
        for name in ("user-setup", "project-setup"):
            text = (SKILL_ROOT / name / "SKILL.md").read_text(encoding="utf-8")
            self.assertIn("--show", text)
            self.assertIn("--summary --output json", text)
            self.assertIn("hidden tool", text.lower())
            self.assertIn("user terminal", text.lower())
            self.assertIn("--input-channel agent", text)
            self.assertNotIn("## Setup Question Sequence", text)
            self.assertNotIn("## Required order", text)
            self.assertIn("you, the AI agent", text)
            self.assertIn("optional", text)
            self.assertNotIn("give this one command", text)
            self.assertNotIn("repeated AI questions", text)
        project = (SKILL_ROOT / "project-setup/SKILL.md").read_text(encoding="utf-8")
        for required in ("--scope user --user-root", "one question per message", "scan before deciding",
                         "ask the user to confirm with your evidence", "--input-channel agent --apply",
                         "No capability file is needed", "never fabricate one", "Never invent roles"):
            self.assertIn(required, " ".join(project.split()))
        self.assertNotIn("--capabilities <trusted-capabilities-file>", project)
        self.assertIn("never skips this interview", " ".join(project.split()))
        self.assertNotIn('"set it up for me"', project)
        self.assertIn("project_state", project)
        user = (SKILL_ROOT / "user-setup/SKILL.md").read_text(encoding="utf-8")
        self.assertIn("one personal preference per message", " ".join(user.split()))
        self.assertIn("never invent them", " ".join(user.split()))
        questions = (SKILL_ROOT / "user-setup/references/questions.md").read_text(encoding="utf-8")
        self.assertIn("--question", questions)
        self.assertNotIn("Enabled (recommended)", questions)
        self.assertNotIn("Expedited setup enables protection", questions)
        for reference in ("advanced-setup.md", "directive-recovery.md", "structured-choices.md"):
            source = (SKILL_ROOT / "project-setup/references" / reference).read_bytes()
            for destination in (PLUGIN_ROOT, TEMPLATE_ROOT, CLAUDE_TEMPLATE_ROOT):
                self.assertEqual((destination / "project-setup/references" / reference).read_bytes(), source)
        recovery = (SKILL_ROOT / "project-setup/references/directive-recovery.md").read_text(encoding="utf-8")
        self.assertIn("exact preview", recovery)
        self.assertIn("Antigravity context recovery is unavailable", recovery)
        self.assertIn("4,096 UTF-8 byte limit", recovery)

    def test_autonomous_workflow_product_body_matches_the_source_project_copy(self) -> None:
        source = (ROOT / ".agents/skills/autonomous-workflow/SKILL.md").read_bytes()
        self.assertEqual((SKILL_ROOT / "autonomous-workflow" / "SKILL.md").read_bytes(), source)
        for destination in (PLUGIN_ROOT, TEMPLATE_ROOT, CLAUDE_TEMPLATE_ROOT):
            self.assertEqual((destination / "autonomous-workflow" / "SKILL.md").read_bytes(), source)
        text = source.decode("utf-8")
        for rule in ("Never stop midway to ask", "Ask once, at the end", "Approval-required actions stay"):
            self.assertIn(rule, text)
        ledger = yaml.safe_load((SKILL_ROOT / "retired-names.yml").read_text(encoding="utf-8"))
        self.assertEqual(ledger["retired_names"]["verified-workflow"], "autonomous-workflow")
        self.assertFalse((SKILL_ROOT / "verified-workflow").exists())
        for root in (PLUGIN_ROOT, TEMPLATE_ROOT, CLAUDE_TEMPLATE_ROOT):
            self.assertFalse((root / "verified-workflow").exists())

    def test_ai_slop_cleaner_has_target_checklists_and_exact_upstream_copies(self) -> None:
        skill = SKILL_ROOT / "ai-slop-cleaner"
        body = (skill / "SKILL.md").read_text(encoding="utf-8")
        header = yaml.safe_load(body.split("---", 2)[1])
        self.assertIn("code-polish", header["description"])
        self.assertIn("say the new name `ai-slop-cleaner`", body)
        for rule in ("Plan first; confirm before editing", "autonomous-workflow", "Review mode"):
            self.assertIn(rule, body)
        self.assertNotIn("http", body, "source links stay out of SKILL.md")
        for name in ("code.md", "writing.md", "design.md", "hive-overrides.md"):
            source = (skill / "references" / name).read_bytes()
            for destination in (PLUGIN_ROOT, TEMPLATE_ROOT, CLAUDE_TEMPLATE_ROOT):
                self.assertEqual((destination / "ai-slop-cleaner/references" / name).read_bytes(), source)
        upstream = ROOT / "harness/upstream"
        for vendored in ("omc-ai-slop-cleaner", "hallmark"):
            projected = skill / "references/upstream" / vendored
            self.assertTrue((projected / "LICENSE").is_file())
            for path in projected.rglob("*"):
                if path.is_file():
                    relative = path.relative_to(projected)
                    self.assertEqual(path.read_bytes(), (upstream / vendored / relative).read_bytes(), str(relative))
                    for destination in (PLUGIN_ROOT, TEMPLATE_ROOT, CLAUDE_TEMPLATE_ROOT):
                        copy = destination / "ai-slop-cleaner/references/upstream" / vendored / relative
                        self.assertEqual(copy.read_bytes(), path.read_bytes(), str(copy))
        ledger = yaml.safe_load((SKILL_ROOT / "retired-names.yml").read_text(encoding="utf-8"))
        self.assertEqual(ledger["retired_names"]["code-polish"], "ai-slop-cleaner")
        self.assertEqual(ledger["retired_names"]["clean-ai-slop"], "ai-slop-cleaner")
        self.assertNotIn("ai-slop-cleaner", ledger["retired_names"])
        self.assertFalse((SKILL_ROOT / "code-polish").exists())

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
        merged = {
            "knowledge-maintain": "knowledge-manage",
            "knowledge-promote": "knowledge-manage",
            "knowledge-transfer": "knowledge-manage",
            "knowledge-scan": "knowledge-capture",
            "run-checkpoint": "run",
            "run-handoff": "run",
            "run-resume": "run",
            "adversarial-judge": "judge",
            "judge-evidence": "judge",
        }
        events = {event["retired_name"]: event for event in ledger["lifecycle"]}
        for old_name, current_name in merged.items():
            self.assertEqual(retired[old_name], current_name)
            self.assertEqual(events[old_name]["removed_in"], "0.16.0")
            self.assertEqual(events[old_name]["replacement"], current_name)
            self.assertEqual(events[old_name]["transition_kind"], "merge")
            self.assertFalse((SKILL_ROOT / old_name).exists())
            body = (SKILL_ROOT / current_name / "SKILL.md").read_text(encoding="utf-8")
            header = yaml.safe_load(body.split("---", 2)[1])
            self.assertIn(old_name, header["description"])
            self.assertIn(f"`{old_name}`", body)
            self.assertIn(f"say the new name `{current_name}`", body)
            for root in (PLUGIN_ROOT, TEMPLATE_ROOT):
                self.assertFalse((root / old_name).exists())
                self.assertEqual(
                    (root / current_name / "SKILL.md").read_bytes(),
                    (SKILL_ROOT / current_name / "SKILL.md").read_bytes(),
                )
        for current_name in set(merged.values()):
            self.assertNotIn(current_name, retired)
        self.assertEqual(
            hashlib.sha256((SKILL_ROOT / "retired-names.yml").read_bytes()).hexdigest(),
            hashlib.sha256((SKILL_ROOT / "retired-names.yml").read_bytes()).hexdigest(),
        )


if __name__ == "__main__":
    unittest.main()
