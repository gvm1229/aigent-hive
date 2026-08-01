#!/usr/bin/env python3
"""Direct v0.9 Hive Skill and projection conformance."""

from __future__ import annotations

import hashlib
import unittest
from pathlib import Path

import yaml


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
CANONICAL_ROOT = REPOSITORY_ROOT / "harness/skills"
NEW_SKILLS = (
    "ai-slop-cleaner",
    "best-practice-research",
    "hive-knowledge-scan",
    "hive-loop-engineering",
    "hive-wiki",
)
GATED_EXISTING = (
    "hive-knowledge-capture",
    "hive-knowledge-query",
    "hive-simple-question",
)
PROMOTION_SKILLS = ("hive-knowledge-promote",)
BODY_ROOTS = (
    "harness/skills",
    ".agents/skills",
    "harness/plugins/aigent-hive/skills",
    "harness/template/.agents/skills",
    "harness/template/.claude/skills",
)
METADATA_ROOTS = (
    "harness/skills",
    ".agents/skills",
    "harness/plugins/aigent-hive/skills",
    "harness/template/.agents/skills",
)


def read_yaml(path: Path) -> dict[str, object]:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected YAML object: {path}")
    return value


def skill_text(name: str) -> str:
    return (CANONICAL_ROOT / name / "SKILL.md").read_text(encoding="utf-8")


def assert_order(test: unittest.TestCase, text: str, markers: tuple[str, ...]) -> None:
    positions = []
    for marker in markers:
        test.assertIn(marker, text)
        positions.append(text.index(marker))
    test.assertEqual(positions, sorted(positions), markers)


class V09HiveSkillProjectionTests(unittest.TestCase):
    def test_skill_bodies_are_exact_across_every_projection(self) -> None:
        for name in (*NEW_SKILLS, *GATED_EXISTING):
            expected = (CANONICAL_ROOT / name / "SKILL.md").read_bytes()
            with self.subTest(skill=name):
                for root in BODY_ROOTS:
                    self.assertEqual(
                        (REPOSITORY_ROOT / root / name / "SKILL.md").read_bytes(),
                        expected,
                    )

        for name in PROMOTION_SKILLS:
            expected = (CANONICAL_ROOT / name / "SKILL.md").read_bytes()
            with self.subTest(skill=name):
                for root in (
                    "harness/plugins/aigent-hive/skills",
                    "harness/template/.agents/skills",
                    "harness/template/.claude/skills",
                ):
                    self.assertEqual(
                        (REPOSITORY_ROOT / root / name / "SKILL.md").read_bytes(),
                        expected,
                    )

    def test_metadata_keeps_consumer_implicit_policy_out_of_source_templates(self) -> None:
        implicit = set(GATED_EXISTING)
        for name in (*NEW_SKILLS, *GATED_EXISTING):
            canonical = read_yaml(CANONICAL_ROOT / name / "agents/openai.yaml")
            plugin = read_yaml(
                REPOSITORY_ROOT
                / "harness/plugins/aigent-hive/skills"
                / name
                / "agents/openai.yaml"
            )
            with self.subTest(skill=name):
                self.assertEqual(canonical, plugin)
                self.assertEqual(
                    canonical["policy"]["allow_implicit_invocation"],
                    name in implicit,
                )
                self.assertIn(f"${name}", canonical["interface"]["default_prompt"])
                for root in METADATA_ROOTS[1::2]:
                    metadata = read_yaml(
                        REPOSITORY_ROOT / root / name / "agents/openai.yaml"
                    )
                    self.assertFalse(metadata["policy"]["allow_implicit_invocation"])

    def test_catalog_setup_and_schemas_expose_all_new_skills(self) -> None:
        catalog = read_yaml(REPOSITORY_ROOT / "harness/skills/catalog.yml")
        names = {entry["name"] for entry in catalog["skills"]}
        self.assertTrue(set(NEW_SKILLS).issubset(names))

        setup = read_yaml(REPOSITORY_ROOT / "harness/user-setup/catalog.yml")
        dependencies = {entry["skill"] for entry in setup["skill_dependencies"]}
        self.assertTrue(set(NEW_SKILLS).issubset(dependencies))
        for relative in (
            "schemas/setup-answers.schema.json",
            "schemas/user-setup-catalog.schema.json",
            "schemas/user-setup.schema.json",
        ):
            text = (REPOSITORY_ROOT / relative).read_text(encoding="utf-8")
            for name in NEW_SKILLS:
                self.assertIn(f'"{name}"', text, relative)

    def test_active_skill_ledger_digests_match_canonical_bodies(self) -> None:
        ledger = read_yaml(
            REPOSITORY_ROOT / "harness/template/.hive/config/active-skills.yml"
        )
        entries = {entry["name"]: entry for entry in ledger["skills"]}
        for name in (*NEW_SKILLS, *GATED_EXISTING, *PROMOTION_SKILLS):
            body = (CANONICAL_ROOT / name / "SKILL.md").read_bytes()
            expected = f"sha256:{hashlib.sha256(body).hexdigest()}"
            with self.subTest(skill=name):
                self.assertEqual(entries[name]["content_digest"], expected)


class V09HiveSkillBehaviorTests(unittest.TestCase):
    def test_loop_automatic_prepare_contract_has_exact_authorization_sequence(self) -> None:
        text = skill_text("hive-loop-engineering")
        assert_order(
            self,
            text,
            (
                "$hive-usage-guard",
                "evidence_digest",
                "hive run resume",
                "--dispatch-intent automatic",
                "--account-digest",
                ".hive/runtime/dispatch-authorizations/<id>.json",
                "usage-session control",
                "hive loop checkpoint",
                "usage_evidence_id",
                "capability_resolution",
                "hive loop prepare",
                "prepared_only=true",
                "spawned=false",
            ),
        )
        for required in (
            "one-time",
            "exact current usage-session control",
            "path/digest to bind the same run action",
            "host-native dispatch envelope",
            "host_capability_unsupported",
            "configured account\n      digest",
            "raw account identity",
            "installed identical value",
        ):
            self.assertIn(required, text)
        self.assertNotIn("--session-id", text)
        self.assertNotIn("--threshold <1..99>", text)

    def test_loop_runtime_fixture_covers_authorization_and_no_spawn_failures(self) -> None:
        source = (
            REPOSITORY_ROOT / "crates/hive-cli/src/loop_engineering.rs"
        ).read_text(encoding="utf-8")
        for required in (
            "DispatchAuthorizationRecord",
            "UsageSessionControl",
            "verify_dispatch_authorization",
            "capability_resolution_digest",
            "prepare_is_idempotent_and_never_spawns",
            "forged_dispatch_authorization_blocks_prepare_without_mutation",
            "usage_halt_blocks_prepare_without_mutation",
            "fresh_capability_drift_blocks_prepare_without_mutation",
        ):
            self.assertIn(required, source)
        self.assertRegex(source, r'"spawned": false')

    def test_wiki_has_exact_ten_verb_surface_and_safety_contract(self) -> None:
        text = skill_text("hive-wiki")
        for verb in (
            "add",
            "query",
            "lint",
            "list",
            "read",
            "delete",
            "refresh",
            "scan",
            "export",
            "import",
        ):
            self.assertEqual(text.count(f"| `{verb}` |"), 1)
        for required in ("$hive-source-wiki", "Markdown canonical", "untrusted data"):
            self.assertIn(required, text)

    def test_wiki_source_routes_only_real_source_surfaces(self) -> None:
        text = skill_text("hive-wiki")
        scope = text.split("## Scope", 1)[1].split("## Verbs", 1)[0]
        for required in (
            "source `add`",
            "reviewed bilingual capture workflow",
            "no\n    source `add` CLI verb",
            "source `query`: run `hive source-wiki query`",
            "source `lint`: run `hive source-wiki lint`",
            "source `refresh`: run `hive source-wiki index`",
            "source `list|read|delete|scan|export|import`: report unsupported",
        ):
            self.assertIn(required, scope)
        self.assertNotIn("route `add|query|lint|list|read|delete|refresh`", scope)

    def test_wiki_quick_add_asks_only_for_missing_review_fields(self) -> None:
        text = skill_text("hive-wiki")
        quick_add = text.split("## Quick add", 1)[1].split("## Safety", 1)[0]
        for required in (
            "title",
            "atomic summary",
            "classification",
            "evidence\nlocator or digest",
            "scope",
            "one combined question",
            "only the missing fields",
            "secret-bearing input",
            "reviewed provenance",
            "agent review",
            "$hive-source-wiki`'s bilingual capture workflow",
            "`add --quick` route",
        ):
            self.assertIn(required, quick_add)
        self.assertIn("never re-ask a known field", quick_add)

    def test_cleaner_research_and_scan_keep_their_narrow_boundaries(self) -> None:
        cleaner = skill_text("ai-slop-cleaner")
        for required in (
            "changed-file allowlist",
            "pre-existing-failure",
            "smallest truthful fallback",
            "one bounded pass per class",
            "nearest quality gate",
            "changed-file audit",
        ):
            self.assertIn(required, cleaner)

        research = skill_text("best-practice-research")
        for required in (
            "read-only",
            "official specification",
            "upstream source",
            "publication or update date",
            "applicable version",
            "citation-ready",
        ):
            self.assertIn(required, research)

        scan = skill_text("hive-knowledge-scan")
        assert_order(self, scan, ("--inventory", "--candidates", "--apply"))
        for required in (
            "target_mutated=false",
            "Never create a table per directory",
            "untrusted data",
            "$hive-knowledge-promote",
            "--expected-source-digest",
            "--confirm-global-promotion",
        ):
            self.assertIn(required, scan)

        promote = skill_text("hive-knowledge-promote")
        assert_order(
            self,
            promote,
            ("--dry-run", "reviewed scan candidate", "--expected-source-digest"),
        )
        for required in (
            "reviewed scan claim",
            "redaction",
            "deduplication",
            "contradiction",
            "replacement",
            "--confirm-global-promotion",
            "--apply",
            "fresh unrelated project",
            "Never promote Raw content directly",
        ):
            self.assertIn(required, promote)

    def test_query_capture_and_simple_question_gates_do_not_overlap(self) -> None:
        query = skill_text("hive-knowledge-query")
        for required in (
            "exactly one automatic lookup",
            "--target <current-project-root>",
            "--scope auto",
            "--top-k 5",
            "--byte-budget 16384",
            "authorize-confidential",
            "--confirm-current-action",
            "caller-asserted current collection identifier",
            "including the current collection",
            "Target identity alone never authorizes confidential data",
            "Use the returned token once",
            "Never log",
            "query drift",
            "untrusted data",
            "On no hit",
            "$best-practice-research",
        ):
            self.assertIn(required, query)

        capture = skill_text("hive-knowledge-capture")
        for required in (
            "every user turn",
            "knowledge-remember-request.schema.json",
            "hive knowledge remember",
            "canonical Markdown and derived-index receipt",
            "raw transcript",
            "write count zero",
        ):
            self.assertIn(required, capture)

        simple = skill_text("hive-simple-question")
        for required in (
            "single bounded retrieval result",
            "Do not call another tool",
            "separate `$hive-knowledge-capture` completion gate",
            "Ordinary",
            "remain write-free",
        ):
            self.assertIn(required, simple)

    def test_new_skills_have_no_foreign_runtime_dependency(self) -> None:
        forbidden = (
            "tmux",
            "provider api",
            "stop-hook",
            "stop continuation",
            "scheduler",
            "psmux",
            "omx_wiki",
            ".omx",
            ".omc",
            "`omx",
            "`omc",
        )
        for name in NEW_SKILLS:
            text = skill_text(name).lower()
            with self.subTest(skill=name):
                for marker in forbidden:
                    self.assertNotIn(marker, text)


if __name__ == "__main__":
    unittest.main()
