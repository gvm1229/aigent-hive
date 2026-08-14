"""Static contracts for the current product-only v0.9 Skill suite."""

from __future__ import annotations

import base64
import json
import re
import struct
import unittest
from pathlib import Path

import yaml
from PIL import Image


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
    "iterative-execution", "team-execution", "multi-goal", "custom-subagent-create",
}


def skill_paths(root: Path) -> set[str]:
    return {entry.parent.name for entry in root.glob("*/SKILL.md")}


class Phase3SchemaContract(unittest.TestCase):
    def test_public_logo_marks_are_centered_without_canvas_changes(self) -> None:
        """The visible mark may move, but each published canvas stays exactly centered."""
        logo_paths = (
            ROOT / "docs/assets/branding/hive-logo-mark.png",
            ROOT / "docs/assets/branding/hive-logo-mark-colored.png",
            ROOT / "harness/plugins/aigent-hive/assets/hive-logo-plugin.png",
        )
        for path in logo_paths:
            with Image.open(path).convert("RGBA") as logo:
                background = logo.getpixel((0, 0))
                if background[3] == 0:
                    bounds = logo.getchannel("A").getbbox()
                else:
                    mask = Image.new("L", logo.size)
                    pixels = logo.load()
                    for y in range(logo.height):
                        for x in range(logo.width):
                            if pixels[x, y] != background:
                                mask.putpixel((x, y), 255)
                    bounds = mask.getbbox()
                self.assertIsNotNone(bounds, path)
                left, top, right, bottom = bounds
                mark_center = ((left + right - 1) / 2, (top + bottom - 1) / 2)
                canvas_center = ((logo.width - 1) / 2, (logo.height - 1) / 2)
                self.assertLessEqual(abs(mark_center[0] - canvas_center[0]), 0.5, path)
                self.assertLessEqual(abs(mark_center[1] - canvas_center[1]), 0.5, path)

    def test_public_html_guides_embed_the_current_logo_and_full_feature_cards(self) -> None:
        logo = (ROOT / "docs/assets/branding/hive-logo-mark.png").read_bytes()
        embedded_logo = re.compile(r'--hive-logo:\s*url\("data:image/png;base64,([^"]+)"\)')
        for relative in ("docs/hive-core-features.ko.html", "docs/hive-install-guide.ko.html"):
            html = (ROOT / relative).read_text(encoding="utf-8")
            match = embedded_logo.search(html)
            self.assertIsNotNone(match, relative)
            self.assertEqual(base64.b64decode(match.group(1)), logo, relative)

        core = (ROOT / "docs/hive-core-features.ko.html").read_text(encoding="utf-8")
        self.assertIn("grid-template-columns: minmax(0, 1fr);", core)
        self.assertEqual(core.count('<article class="card">'), 8)
        self.assertEqual(core.count('<div class="use-case">'), 8)

    def test_codex_plugin_uses_named_developer_and_cropped_hive_logo(self) -> None:
        plugin_root = ROOT / "harness/plugins/aigent-hive"
        manifest = json.loads(
            (plugin_root / ".codex-plugin/plugin.json").read_text(encoding="utf-8")
        )
        self.assertEqual(manifest["author"]["name"], "Hojin (Tom) Jeong")
        self.assertEqual(
            manifest["interface"]["developerName"], "Hojin (Tom) Jeong"
        )
        self.assertEqual(manifest["interface"]["brandColor"], "#FFB52E")

        expected_path = "./assets/hive-logo-plugin.png"
        self.assertEqual(manifest["interface"]["logo"], expected_path)
        self.assertEqual(manifest["interface"]["composerIcon"], expected_path)
        logo = (plugin_root / expected_path.removeprefix("./")).read_bytes()
        self.assertEqual(logo[:8], b"\x89PNG\r\n\x1a\n")
        self.assertEqual(struct.unpack(">II", logo[16:24]), (512, 512))

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
    def test_source_and_consumer_language_contracts_keep_the_same_rules(self) -> None:
        source_behavior = (ROOT / ".agents/directives/01-behavior.md").read_text(
            encoding="utf-8"
        )
        source_style = (ROOT / ".agents/directives/08-human-documentation-style.md").read_text(
            encoding="utf-8"
        )
        consumer_template = (ROOT / "harness/template/AGENTS.md.jinja").read_text(
            encoding="utf-8"
        )
        project_base = (
            ROOT / "harness/project-bases/0.9.0/AGENTS.md.template"
        ).read_text(encoding="utf-8")

        for text in (source_behavior, source_style, consumer_template, project_base):
            normalized = " ".join(text.split())
            self.assertIn("ASD-STE100 Simplified Technical English", text)
            self.assertIn(
                "Translate meaning rather than English word order.", normalized
            )
            self.assertIn("mixed Korean-English compounds", normalized)

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

    def test_wiki_lint_routes_source_and_consumer_targets_without_skipping(self) -> None:
        capture = (SKILLS / "knowledge-capture/SKILL.md").read_text(encoding="utf-8")
        maintain = (SKILLS / "knowledge-maintain/SKILL.md").read_text(encoding="utf-8")
        for text in (capture, maintain):
            normalized = " ".join(text.split())
            self.assertIn("hive knowledge lint --target", normalized)
            self.assertIn(
                "hive knowledge lint --target <user-root> --user-root <user-root>",
                normalized,
            )
            self.assertIn("hive source-wiki lint", normalized)
            self.assertIn("hive-source.json", normalized)
            self.assertIn("unregistered", normalized)
        self.assertIn("never skips lint", " ".join(capture.split()))
        self.assertIn("never a reason to skip Wiki lint", " ".join(maintain.split()))

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
        self.assertIn("installed product `hive usage`", agents)
        self.assertIn("sole usage-policy authority", agents)
        self.assertIn("Never start a source watcher", agents)

    def test_source_directives_continue_agent_owned_work_until_closure(self) -> None:
        behavior = (ROOT / ".agents/directives/01-behavior.md").read_text(encoding="utf-8")
        state = (ROOT / ".agents/directives/04-documentation-state.md").read_text(encoding="utf-8")
        session = (ROOT / ".agents/directives/06-session-coordination.md").read_text(encoding="utf-8")
        fixture = json.loads(
            (ROOT / "tests/fixtures/agent-autonomous-continuation.json").read_text(
                encoding="utf-8"
            )
        )
        for required in (
            "all todos",
            "until completion",
            "A progress report that identifies a remaining agent-owned action must not end the task.",
            "unpublished authorized release is work to continue",
            "awaiting-user-authority",
            "awaiting-external-evidence",
        ):
            self.assertIn(required, behavior)
        self.assertIn("Final Response Closure Gate", state)
        self.assertIn("Continue execution when any `agent-owned` item remains", state)
        self.assertIn("Remaining Agent-Owned Actions", session)
        self.assertIn("An `active` manifest prohibits a final completion claim", session)
        self.assertEqual(fixture["terminal_instruction"], "Proceed until all todos are complete.")
        self.assertEqual(fixture["expected_state_before_actions"], "active")
        self.assertEqual(
            fixture["agent_owned_actions"],
            ["fix", "verify", "push", "candidate", "publish"],
        )
        self.assertEqual(fixture["allowed_final_status_before_actions"], None)

    def test_consumer_directives_continue_agent_owned_work_until_closure(self) -> None:
        template = (ROOT / "harness/template/AGENTS.md.jinja").read_text(encoding="utf-8")
        harness = (ROOT / "harness/directives/00-project-harness.md").read_text(
            encoding="utf-8"
        )
        renderer = (ROOT / "crates/hive-render/src/lib.rs").read_text(encoding="utf-8")
        user_install = (ROOT / "crates/hive-cli/src/user_install.rs").read_text(
            encoding="utf-8"
        )
        user_setup = (ROOT / "crates/hive-cli/src/user_setup.rs").read_text(encoding="utf-8")
        for text in (template, harness, renderer):
            for required in (
                "all todos",
                "until completion",
                "A progress report naming such work must not end the task.",
                "awaiting-user-authority",
                "awaiting-external-evidence",
            ):
                self.assertIn(required, text)
        for required in (
            "Before a final response, classify every remaining item",
            "`all todos`, `until completion`, `do not stop` 또는 같은 완료 요청",
            "`agent-owned` 작업 `0건`일 때만 완료 표기",
        ):
            self.assertIn(required, user_install)
            self.assertIn(required, user_setup)

    def test_global_setup_contract_uses_describe_progress_and_conditional_integrations(self) -> None:
        skill = (SKILLS / "user-setup/SKILL.md").read_text(encoding="utf-8")
        for required in (
            "hive setup --scope user --describe --output json",
            "Get-Command hive",
            "where.exe hive",
            "npm prefix -g",
            "hive setup --progress save --scope user",
            "활성화 (권장)",
            "hive usage probe-native --host codex --output json",
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

    def test_projection_refresh_purges_only_authenticated_retired_hive_skills(self) -> None:
        user_setup = (ROOT / "crates/hive-cli/src/user_setup.rs").read_text(encoding="utf-8")
        project_upgrade = (ROOT / "crates/hive-cli/src/project_upgrade.rs").read_text(
            encoding="utf-8"
        )
        user_setup_skill = (SKILLS / "user-setup/SKILL.md").read_text(encoding="utf-8")
        project_refresh = (SKILLS / "project-refresh/SKILL.md").read_text(
            encoding="utf-8"
        )
        for required in (
            "authenticated_retired_user_skill_files",
            "retired_builtin_skill_names",
            "historical_builtin_skills",
            "prune_user_setup_empty_ancestors",
            "three_way_merge_hive_directive",
        ):
            self.assertIn(required, user_setup)
        self.assertIn("three_way_merge_hive_directive", project_upgrade)
        self.assertIn("retired-name", user_setup_skill)
        self.assertIn("retired Hive Skill", project_refresh)

    def test_new_universal_skill_boundaries_are_present(self) -> None:
        ship = (SKILLS / "ship/SKILL.md").read_text(encoding="utf-8")
        amend = (SKILLS / "amend-directive/SKILL.md").read_text(encoding="utf-8")
        self.assertIn("explicit", ship.lower())
        self.assertIn("foreign", amend.lower())
        self.assertIn("signed", amend.lower())


if __name__ == "__main__":
    unittest.main()
