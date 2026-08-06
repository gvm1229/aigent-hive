#!/usr/bin/env python3
"""Static shipped-contract gates for Phase 3 portable Skills."""

from __future__ import annotations

import hashlib
import json
import re
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
    "ai-slop-cleaner",
    "best-practice-research",
    "setup-hive",
    "setup-harness",
    "auto-setup-harness",
    "hive-simple-question",
    "hive-prompt-refine",
    "hive-knowledge-capture",
    "hive-knowledge-promote",
    "hive-knowledge-query",
    "hive-knowledge-scan",
    "hive-loop-engineering",
    "hive-knowledge-maintenance",
    "hive-run-checkpoint",
    "hive-run-resume",
    "hive-role-handoff",
    "hive-judge-package",
    "hive-update",
    "hive-usage-guard",
    "hive-migrate",
    "hive-project-upgrade",
    "hive-wiki",
}
IMPLEMENTED_SKILLS = {
    "ai-slop-cleaner",
    "best-practice-research",
    "setup-hive",
    "setup-harness",
    "auto-setup-harness",
    "hive-simple-question",
    "hive-prompt-refine",
    "hive-knowledge-capture",
    "hive-knowledge-promote",
    "hive-knowledge-query",
    "hive-knowledge-scan",
    "hive-loop-engineering",
    "hive-knowledge-maintenance",
    "hive-run-checkpoint",
    "hive-run-resume",
    "hive-role-handoff",
    "hive-judge-package",
    "hive-update",
    "hive-usage-guard",
    "hive-migrate",
    "hive-project-upgrade",
    "hive-wiki",
}
CATALOG_ONLY_SKILLS = BUILTIN_SKILLS - IMPLEMENTED_SKILLS
IMPLICIT_PLUGIN_SKILLS = {
    "setup-hive",
    "setup-harness",
    "auto-setup-harness",
    "hive-simple-question",
    "hive-knowledge-capture",
    "hive-knowledge-query",
    "hive-prompt-refine",
    "hive-usage-guard",
}
IMPLICIT_DESCRIPTION_BUDGET = 1_800


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
    def test_source_directives_preserve_valid_knowledge_during_simplification(self) -> None:
        manifest = (REPOSITORY_ROOT / "AGENTS.md").read_text(encoding="utf-8")
        editing = (
            REPOSITORY_ROOT / ".agents/directives/00-editing-discipline.md"
        ).read_text(encoding="utf-8")
        documentation = (
            REPOSITORY_ROOT / ".agents/directives/04-documentation-state.md"
        ).read_text(encoding="utf-8")
        safety = (
            REPOSITORY_ROOT / ".agents/directives/05-security-safety.md"
        ).read_text(encoding="utf-8")

        self.assertIn("Move it to the smallest fitting canonical document", manifest)
        self.assertIn("Inventory the durable claims", editing)
        self.assertIn("Map every removed durable claim", editing)
        self.assertIn("verify the exact replacement locator", documentation)
        self.assertIn("README streamlining is not deletion authority", safety)

    def test_source_auto_setup_projection_matches_harness_canonical_bytes(self) -> None:
        canonical = (
            SKILL_ROOT / "auto-setup-harness/SKILL.md"
        ).read_bytes()
        source_projection = (
            REPOSITORY_ROOT
            / ".agents/skills/auto-setup-harness/SKILL.md"
        ).read_bytes()
        self.assertEqual(source_projection, canonical)

    def test_auto_setup_infers_and_asks_only_unresolved_fields(self) -> None:
        skill = (
            SKILL_ROOT / "auto-setup-harness/SKILL.md"
        ).read_text(encoding="utf-8")
        for required in (
            "confidence as `explicit`, `strong`, or `unresolved`",
            "Set `setup_mode: expedited`",
            "Ask only unresolved questions",
            "Default root-knowledge promotion categories to empty",
            "Approve no optional third-party Skill and no fallback hook by inference",
            "Zero-Question Gate",
        ):
            with self.subTest(required=required):
                self.assertIn(required, skill)

    def test_setup_hive_expedited_defaults_are_fixed(self) -> None:
        skill = (SKILL_ROOT / "setup-hive/SKILL.md").read_text(
            encoding="utf-8"
        )
        for required in (
            "Expedited — set everything to default",
            "ask for interface language first",
            "Interface language: the language already selected by the user",
            "Daily update check: the explicit answer already selected by the user",
            "retries on the next host session after offline failure",
            "never installs an update",
            "Wiki: enabled with the selected interface language",
            "Agent persona: `strict`",
            "every built-in Skill in the signed catalog",
            "Usage guard: disabled",
        ):
            with self.subTest(required=required):
                self.assertIn(required, skill)

    def test_setup_scope_routing_keeps_global_and_project_work_disjoint(self) -> None:
        global_skill = (SKILL_ROOT / "setup-hive/SKILL.md").read_text(
            encoding="utf-8"
        )
        project_skill = (SKILL_ROOT / "setup-harness/SKILL.md").read_text(
            encoding="utf-8"
        )
        for required in (
            "## Scope Routing",
            "bare request to set up, install, configure, or reconfigure Hive",
            "Do not inspect an ambient working directory",
            "Never create, preview, or apply a project harness",
        ):
            with self.subTest(global_requirement=required):
                self.assertIn(required, global_skill)
        for required in (
            "only when the user identifies a project, repository, folder, path",
            "Those requests belong to `setup-hive`",
            "separate confirmation before project inspection, preview, or apply",
        ):
            with self.subTest(project_requirement=required):
                self.assertIn(required, project_skill)

    def test_source_prompt_refine_projection_matches_harness_canonical_bytes(self) -> None:
        canonical = (
            SKILL_ROOT / "hive-prompt-refine/SKILL.md"
        ).read_bytes()
        source_projection = (
            REPOSITORY_ROOT
            / ".agents/skills/hive-prompt-refine/SKILL.md"
        ).read_bytes()
        self.assertEqual(source_projection, canonical)

    def test_source_prompt_quality_gate_automatically_refines_material_ambiguity(self) -> None:
        directive = (
            REPOSITORY_ROOT / ".agents/directives/01-behavior.md"
        ).read_text(encoding="utf-8")
        source_manifest = (REPOSITORY_ROOT / "AGENTS.md").read_text(
            encoding="utf-8"
        )
        for required in (
            "automatically load `hive-prompt-refine`",
            "`awaiting-approval`",
            "project read, tool, write, network, subagent, run, memory capture, and execution",
            "sufficiently clear ordinary task, simple or editless question",
        ):
            with self.subTest(required=required):
                self.assertIn(required, directive)
        self.assertIn("prompt quality gate", source_manifest)
        self.assertIn("materially ambiguous ordinary work route", source_manifest)
        self.assertIn("awaiting-approval", source_manifest)

    def test_plan_backed_goal_reconciles_every_checklist_before_execution(
        self,
    ) -> None:
        directive = (
            REPOSITORY_ROOT / ".agents/directives/04-documentation-state.md"
        ).read_text(encoding="utf-8")
        source_manifest = (REPOSITORY_ROOT / "AGENTS.md").read_text(
            encoding="utf-8"
        )

        for requirement in (
            "Before starting or resuming any goal backed by `docs/plans/PLAN.md`",
            "Load every fragment listed under the index's `Active fragments` section",
            "Inspect every active checklist item, not only the unchecked subset.",
            "Mark every already-proven unchecked item complete in its single owning active fragment",
            "Only after this reconciliation, derive the remaining execution queue",
            "mandatory on every PLAN-backed goal start or resume",
            "An unchecked box is not proof that work remains",
            "Derive the `PLAN.md` completion index only from Phase milestone checklists",
            "Update the completion index in the same edit as any counted checklist state change",
            "Whenever a plan is created or materially revised to govern repository work",
            "canonical tracked plan set before executing that plan.",
            "must never be its sole authority",
            "Legacy native goal wording that refers to unchecked items in `docs/plans/PLAN.md` must resolve to the documents listed under `Active fragments`",
            "the intentional absence of checkboxes in the compact index is not completion evidence",
        ):
            self.assertIn(requirement, directive)
        self.assertIn(
            "before starting or resuming any `docs/plans/PLAN.md`-backed goal",
            source_manifest,
        )

    def test_source_commit_workflow_splits_independent_concerns(self) -> None:
        directive = (
            REPOSITORY_ROOT / ".agents/directives/03-workflow.md"
        ).read_text(encoding="utf-8")
        guide = (
            REPOSITORY_ROOT / "docs/guides/commit-rules.md"
        ).read_text(encoding="utf-8")
        skill = (
            REPOSITORY_ROOT / ".agents/skills/hive-commit/SKILL.md"
        ).read_text(encoding="utf-8")
        metadata = (
            REPOSITORY_ROOT
            / ".agents/skills/hive-commit/agents/openai.yaml"
        ).read_text(encoding="utf-8")

        for requirement in (
            "independently reviewable and revertible intent",
            "A Wiki capture and a product version or release-date change are separate commits.",
            "use patch staging or sequence the edits",
            "Do not rewrite an existing commit solely to apply current commit-splitting policy",
            "ordinary fast-forward direct pushes",
            "Push ordinary verified commits directly to `develop`",
            "status checks for this branch.",
            "A created `staging` branch must use a strict ruleset",
        ):
            self.assertIn(requirement, directive)
        for requirement in (
            "독립 검토·독립 되돌리기 가능한 의도",
            "Wiki 기록과 `hive --version` 변경의 별도 커밋",
            "과거 커밋 소급 적용 금지",
        ):
            self.assertIn(requirement, guide)
        for requirement in (
            "Build a concern map before staging.",
            "Stage one concern only.",
            "Do not rewrite, amend, rebase, or split existing history",
            "Never bypass repository hooks",
        ):
            self.assertIn(requirement, skill)
        self.assertIn("allow_implicit_invocation: true", metadata)

    def test_source_directive_amend_skill_has_bounded_explicit_authority(self) -> None:
        skill = (
            REPOSITORY_ROOT
            / ".agents/skills/hive-directive-amend/SKILL.md"
        ).read_text(encoding="utf-8")
        metadata = (
            REPOSITORY_ROOT
            / ".agents/skills/hive-directive-amend/agents/openai.yaml"
        ).read_text(encoding="utf-8")

        for requirement in (
            "$hive-directive-amend [--source|--consumer] <amendment command>",
            "Stop the command at the first line break.",
            "No flag: amend both source-development and consumer-product directives.",
            "`--source`: amend source-development directives only.",
            "`--consumer`: amend consumer-product directives only.",
            "Keep this Skill source-only under `.agents/skills/`.",
            "Never edit generated output alone when a canonical producer exists.",
        ):
            self.assertIn(requirement, skill)
        self.assertIn("allow_implicit_invocation: false", metadata)
        self.assertIn("$hive-directive-amend", metadata)

    def test_source_consumer_skill_reuse_and_orchestration_independence(self) -> None:
        source_manifest = (REPOSITORY_ROOT / "AGENTS.md").read_text(
            encoding="utf-8"
        )
        architecture = (
            REPOSITORY_ROOT / ".agents/directives/02-architecture.md"
        ).read_text(encoding="utf-8")
        plan = (
            REPOSITORY_ROOT / "docs/plans/active/source-docs-wiki.md"
        ).read_text(encoding="utf-8")

        for requirement in (
            "Reuse useful Hive-owned Skills bidirectionally",
            "shared Skill is canonical under `harness/skills/`",
            "Never import installed consumer state, runtime data, or user knowledge into source.",
        ):
            self.assertIn(requirement, source_manifest)
        for requirement in (
            "Do not select, invoke, install, or configure OMX/OMC for a new workflow",
            "foreign read-only provenance",
            "Permit bidirectional reuse only for Hive-owned Skill source",
            "Do not use `omx_wiki/`, `.omx/wiki/`, or consumer `.hive/knowledge/`",
        ):
            self.assertIn(requirement, architecture)
        for requirement in (
            "`docs/facts/en/`·`docs/facts/ko/`",
            "`hive-wiki` Markdown parser·lint·SQLite rebuild·query",
            "one-H1·no-subsection·800-byte atomic body schema",
        ):
            self.assertIn(requirement, plan)

    def test_plan_index_is_compact_and_active_checklist_ids_are_unique(self) -> None:
        plan_root = REPOSITORY_ROOT / "docs/plans"
        index = (plan_root / "PLAN.md").read_text(encoding="utf-8")
        self.assertLess(len(index.encode("utf-8")), 8_192)
        self.assertNotRegex(index, r"(?m)^- \[[ x]\] ")
        self.assertIn(
            "“unchecked item in `docs/plans/PLAN.md`”는 `PLAN.md` 내부 checkbox가 아니라 아래 `Active fragments`의 unchecked item을 뜻함",
            index,
        )

        linked_fragments = {
            match
            for match in re.findall(r"\]\(([^)]+\.md)\)", index)
            if not match.startswith("../")
        }
        expected_fragments = {
            "active/documentation-style.md",
            "active/bootstrap-global-setup-recovery.md",
            "active/docs-wiki-migration.md",
            "active/model-routed-custom-subagents.md",
            "active/native-usage-sensor.md",
            "active/native-iterative-execution.md",
            "active/plugin-project-lifecycle.md",
            "active/prompt-refine-auto-routing.md",
            "active/release-0.8.0.md",
            "active/release-0.9.0.md",
            "active/security-review.md",
            "active/source-docs-wiki.md",
            "active/test-release-setup-routing.md",
            "active/user-onboarding-shared-index.md",
            "active/v0.9.0-global-knowledge-rag.md",
            "active/v0.9.0-knowledge-portability-scan.md",
            "active/v0.9.0-loop-wiki-skills.md",
            "active/v0.9.0-test-finalization.md",
            "active/windows-shell-install.md",
            "contracts/README.md",
            "phases/07-public-qualification.md",
            "phases/README.md",
            "references.md",
            "stages/README.md",
        }
        self.assertEqual(linked_fragments, expected_fragments)
        for relative in linked_fragments:
            with self.subTest(fragment=relative):
                self.assertTrue((plan_root / relative).is_file())

        active_fragments = [
            plan_root / "active/documentation-style.md",
            plan_root / "active/bootstrap-global-setup-recovery.md",
            plan_root / "active/docs-wiki-migration.md",
            plan_root / "active/model-routed-custom-subagents.md",
            plan_root / "active/native-usage-sensor.md",
            plan_root / "active/native-iterative-execution.md",
            plan_root / "active/plugin-project-lifecycle.md",
            plan_root / "active/prompt-refine-auto-routing.md",
            plan_root / "active/release-0.8.0.md",
            plan_root / "active/release-0.9.0.md",
            plan_root / "active/security-review.md",
            plan_root / "active/source-docs-wiki.md",
            plan_root / "active/test-release-setup-routing.md",
            plan_root / "active/user-onboarding-shared-index.md",
            plan_root / "active/v0.9.0-global-knowledge-rag.md",
            plan_root / "active/v0.9.0-knowledge-portability-scan.md",
            plan_root / "active/v0.9.0-loop-wiki-skills.md",
            plan_root / "active/v0.9.0-test-finalization.md",
            plan_root / "active/windows-shell-install.md",
            plan_root / "phases/07-public-qualification.md",
        ]
        self.assertEqual(
            {path.relative_to(plan_root).as_posix() for path in active_fragments},
            {
                "active/documentation-style.md",
                "active/bootstrap-global-setup-recovery.md",
                "active/docs-wiki-migration.md",
                "active/model-routed-custom-subagents.md",
                "active/native-usage-sensor.md",
                "active/native-iterative-execution.md",
                "active/plugin-project-lifecycle.md",
                "active/prompt-refine-auto-routing.md",
                "active/release-0.8.0.md",
                "active/release-0.9.0.md",
                "active/security-review.md",
                "active/source-docs-wiki.md",
                "active/test-release-setup-routing.md",
                "active/user-onboarding-shared-index.md",
                "active/v0.9.0-global-knowledge-rag.md",
                "active/v0.9.0-knowledge-portability-scan.md",
                "active/v0.9.0-loop-wiki-skills.md",
                "active/v0.9.0-test-finalization.md",
                "active/windows-shell-install.md",
                "phases/07-public-qualification.md",
            },
        )
        seen_ids: set[str] = set()
        for fragment in active_fragments:
            for line in fragment.read_text(encoding="utf-8").splitlines():
                if not re.match(r"^- \[[ x]\] ", line):
                    continue
                match = re.match(r"^- \[[ x]\] \[([A-Z0-9-]+)\] ", line)
                self.assertIsNotNone(match, f"missing checklist ID: {fragment}:{line}")
                assert match is not None
                checklist_id = match.group(1)
                self.assertNotIn(checklist_id, seen_ids)
                seen_ids.add(checklist_id)
        self.assertTrue(seen_ids)

        phase_fragments = {
            path.name for path in (plan_root / "phases").glob("[0-9][0-9]-*.md")
        }
        self.assertEqual(
            phase_fragments,
            {
                "00-source-bootstrap.md",
                "01-setup-renderer.md",
                "02-knowledge-index.md",
                "03-skills-projection.md",
                "04-role-run-interoperability.md",
                "05-usage-judge.md",
                "06-update-migration-release.md",
                "07-public-qualification.md",
            },
        )
        stage_fragments = {
            path.name for path in (plan_root / "stages").glob("[0-9][0-9]*-*.md")
        }
        self.assertEqual(
            stage_fragments,
            {
                "00-entry-routing.md",
                "01a-setup-discovery-consent.md",
                "01b-setup-rendering-contract.md",
                "02-harness-ownership.md",
                "03-simple-question-isolation.md",
                "04-prompt-refine.md",
                "05-roles-orchestration.md",
                "06-durable-run-completion.md",
                "07-usage-guard.md",
                "08-verification-judge.md",
                "09-knowledge-memory.md",
                "10-completion-resume.md",
                "11-update-migration.md",
            },
        )
        self.assertFalse((plan_root / "stages/workflows.md").exists())
        self.assertFalse((plan_root / "history/milestones.md").exists())

        def checklist_counts(paths: list[Path]) -> tuple[int, int]:
            text = "\n".join(path.read_text(encoding="utf-8") for path in paths)
            return (
                len(re.findall(r"(?m)^- \[x\] ", text)),
                len(re.findall(r"(?m)^- \[ \] ", text)),
            )

        completed_phase_paths = sorted(
            (plan_root / "phases").glob("0[0-6]-*.md")
        )
        phase_7_path = plan_root / "phases/07-public-qualification.md"
        documentation_path = plan_root / "active/documentation-style.md"
        bootstrap_recovery_path = (
            plan_root / "active/bootstrap-global-setup-recovery.md"
        )
        docs_wiki_path = plan_root / "active/docs-wiki-migration.md"
        model_routed_path = (
            plan_root / "active/model-routed-custom-subagents.md"
        )
        native_usage_path = plan_root / "active/native-usage-sensor.md"
        native_iterative_path = plan_root / "active/native-iterative-execution.md"
        plugin_project_path = plan_root / "active/plugin-project-lifecycle.md"
        prompt_refine_path = plan_root / "active/prompt-refine-auto-routing.md"
        release_09_path = plan_root / "active/release-0.9.0.md"
        test_routing_path = plan_root / "active/test-release-setup-routing.md"
        test_finalization_path = (
            plan_root / "active/v0.9.0-test-finalization.md"
        )
        security_review_path = plan_root / "active/security-review.md"
        source_wiki_path = plan_root / "active/source-docs-wiki.md"
        onboarding_path = plan_root / "active/user-onboarding-shared-index.md"
        v09_rag_path = plan_root / "active/v0.9.0-global-knowledge-rag.md"
        v09_portability_path = (
            plan_root / "active/v0.9.0-knowledge-portability-scan.md"
        )
        v09_skill_path = plan_root / "active/v0.9.0-loop-wiki-skills.md"
        windows_shell_path = plan_root / "active/windows-shell-install.md"
        progress_rows = (
            ("Phase 0–6", *checklist_counts(completed_phase_paths)),
            ("Phase 7", *checklist_counts([phase_7_path])),
            (
                "User plugin/project lifecycle",
                *checklist_counts([plugin_project_path]),
            ),
            (
                "Host-native usage sensors",
                *checklist_counts([native_usage_path]),
            ),
            (
                "Global onboarding·shared index",
                *checklist_counts([onboarding_path]),
            ),
            (
                "Source docs Wiki",
                *checklist_counts([source_wiki_path]),
            ),
            (
                "Windows shell install boundary",
                *checklist_counts([windows_shell_path]),
            ),
            ("문서 말투", *checklist_counts([documentation_path])),
            ("Security review", *checklist_counts([security_review_path])),
            ("Docs Wiki migration", *checklist_counts([docs_wiki_path])),
            (
                "v0.9 loop·Wiki·Skill suite",
                *checklist_counts([v09_skill_path]),
            ),
            (
                "v0.9 global knowledge RAG",
                *checklist_counts([v09_rag_path]),
            ),
            (
                "v0.9 knowledge portability·scan",
                *checklist_counts([v09_portability_path]),
            ),
            (
                "Hive-native 반복 실행",
                *checklist_counts([native_iterative_path]),
            ),
            (
                "Model-routed custom subagent",
                *checklist_counts([model_routed_path]),
            ),
            (
                "Prompt refine 자동 routing",
                *checklist_counts([prompt_refine_path]),
            ),
            (
                "v0.9 test 기능 마감",
                *checklist_counts([test_finalization_path]),
            ),
            (
                "v0.9 full release",
                *checklist_counts([release_09_path]),
            ),
            (
                "Test release setup routing",
                *checklist_counts([test_routing_path]),
            ),
            (
                "Bootstrap·user projection recovery",
                *checklist_counts([bootstrap_recovery_path]),
            ),
        )
        total_done = sum(row[1] for row in progress_rows)
        total_open = sum(row[2] for row in progress_rows)
        for label, done, open_items in progress_rows:
            percentage = 100 * done / (done + open_items)
            rendered_percentage = f"{percentage:.1f}".rstrip("0").rstrip(".")
            self.assertIn(
                f"| {label} | {done} | {open_items} | {rendered_percentage}% |",
                index,
            )
        total_percentage = 100 * total_done / (total_done + total_open)
        self.assertIn(
            f"| **Canonical total** | **{total_done}** | **{total_open}** | "
            f"**{total_percentage:.1f}%** |",
            index,
        )

        for fragment in plan_root.rglob("*.md"):
            text = fragment.read_text(encoding="utf-8")
            self.assertLess(
                len(text.encode("utf-8")),
                8_192,
                f"plan fragment exceeds 8 KiB: {fragment.relative_to(plan_root)}",
            )
            if fragment not in active_fragments:
                self.assertNotRegex(text, r"(?m)^- \[ \] ")
            for target in re.findall(r"\]\(([^)]+)\)", text):
                path = target.split("#", 1)[0]
                if not path or "://" in path:
                    continue
                self.assertTrue(
                    (fragment.parent / path).exists(),
                    f"broken plan link: {fragment.relative_to(plan_root)} -> {target}",
                )

    def test_v09_knowledge_portability_plan_has_bounded_nonoverlap(self) -> None:
        plan = (
            REPOSITORY_ROOT
            / "docs/plans/active/v0.9.0-knowledge-portability-scan.md"
        ).read_text(encoding="utf-8")
        research = (
            REPOSITORY_ROOT
            / "docs/research/knowledge-portability-ingestion-retrieval.md"
        ).read_text(encoding="utf-8")
        for requirement in (
            "SQLite·WAL·SHM",
            "directory별 table",
            "collection_id",
            "hive-knowledge-scan",
            "기존 `hive-knowledge-query`",
            "untrusted data",
            "detached",
        ):
            with self.subTest(requirement=requirement):
                self.assertIn(requirement, plan)
        self.assertIn("사용자 결정 잔여: 없음", research)

    def test_prompt_refine_auto_routing_plan_requires_approval_stop(
        self,
    ) -> None:
        plan_root = REPOSITORY_ROOT / "docs/plans"
        fragment = (
            plan_root / "active/prompt-refine-auto-routing.md"
        ).read_text(encoding="utf-8")
        adr = (
            REPOSITORY_ROOT
            / "docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md"
        ).read_text(encoding="utf-8")
        current = (
            REPOSITORY_ROOT / "docs/state/CURRENT.md"
        ).read_text(encoding="utf-8")

        for required in (
            "Material ambiguity",
            "awaiting-approval",
            "`$hive-prompt-refine --run <payload>`",
            "Simple question·editless question·clear work",
            "Prompt 분류용 hook·provider API·hidden rewrite 0건",
        ):
            with self.subTest(required=required):
                self.assertIn(required, fragment)
        self.assertIn("Prompt quality gate", adr)
        self.assertIn("material ambiguity 자동 `refine-only`", current)

    def test_native_usage_sensor_plan_demotes_codexbar_for_all_hosts(
        self,
    ) -> None:
        fragment = (
            REPOSITORY_ROOT / "docs/plans/active/native-usage-sensor.md"
        ).read_text(encoding="utf-8")
        research = (
            REPOSITORY_ROOT
            / "docs/research/codex-app-server-usage-sensor.md"
        ).read_text(encoding="utf-8")
        decisions = (
            REPOSITORY_ROOT / "docs/decisions/product-release-decisions.md"
        ).read_text(encoding="utf-8")
        claude_research = (
            REPOSITORY_ROOT
            / "docs/research/claude-code-native-usage-sensor.md"
        ).read_text(encoding="utf-8")
        antigravity_research = (
            REPOSITORY_ROOT
            / "docs/research/antigravity-native-usage-sensor.md"
        ).read_text(encoding="utf-8")
        adr = (
            REPOSITORY_ROOT
            / "docs/decisions/ADR-0010-native-first-usage-sensors.md"
        ).read_text(encoding="utf-8")

        for requirement in (
            "`active-host native → CodexBar`",
            "Native unavailable·unsupported·malformed일 때만 CodexBar fallback",
            "autonomous CodexBar 설치 동의 요청",
            "거절 시 core 기능 유지와 automatic dispatch `usage_unknown`",
            "Qualified native provider의 success·limited에서 CodexBar 0회",
            "CodexBar 분류: 모든 provider에서 fallback-only",
            "`~/.claude/settings.json` mutation 0회",
            "`native=unsupported`",
        ):
            with self.subTest(requirement=requirement):
                self.assertIn(requirement, fragment)
        self.assertIn("account/rateLimits/read", research)
        self.assertIn("Native normalized usage", research)
        self.assertIn("Claude Code status-line JSON capture", decisions)
        self.assertIn("CodexBar는 세 provider 모두", decisions)
        self.assertIn("rate_limits.five_hour.used_percentage", claude_research)
        self.assertIn("Existing status line 자동 교체", claude_research)
        self.assertIn("Interactive TUI panel", antigravity_research)
        self.assertIn("Undocumented local LSP/HTTP", antigravity_research)
        self.assertIn("CodexBar: 세 provider 모두 fallback-only", adr)
        self.assertIn("Native limited 판정 뒤 CodexBar 호출 0회", adr)

    def test_rag_trust_receipt_has_exact_canonical_ownership(self) -> None:
        manifest = (REPOSITORY_ROOT / "harness/manifest.toml").read_text(
            encoding="utf-8"
        )
        receipt = (
            'pattern = ".hive/config/rag-trust.json"\n'
            'ownership = "canonical-data-protected"\n'
            'source = "rag-generation-receipt"'
        )
        self.assertEqual(manifest.count(receipt), 1)
        self.assertNotIn('pattern = ".hive/config/**"', manifest)

    def test_full_editing_discipline_is_exact_and_highest_priority(self) -> None:
        expected_digest = (
            "2445eeaa461ac04d9a5919a9d5499dac6cbe6300f8b57e3ab00215fbd5426fd9"
        )
        source_path = (
            REPOSITORY_ROOT / ".agents/directives/00-editing-discipline.md"
        )
        product_path = (
            REPOSITORY_ROOT
            / "harness/template/.hive/directives/00-editing-discipline.md"
        )
        source_bytes = source_path.read_bytes()
        product_bytes = product_path.read_bytes()
        self.assertEqual(source_bytes, product_bytes)
        self.assertEqual(hashlib.sha256(source_bytes).hexdigest(), expected_digest)
        manifest = (REPOSITORY_ROOT / "harness/manifest.toml").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            'pattern = ".hive/directives/00-editing-discipline.md"\n'
            'ownership = "hive-managed-config"\n'
            'source = "template"',
            manifest,
        )
        source_text = source_bytes.decode("utf-8")
        for sentinel in (
            "# CLAUDE.md",
            "## 1. Think Before Coding",
            "## 2. Simplicity First",
            "## 3. Surgical Changes",
            "## 4. Goal-Driven Execution",
            "Every changed line should trace directly to the user's request.",
            "Strong success criteria let you loop independently.",
        ):
            self.assertIn(sentinel, source_text)

        source_manifest = (REPOSITORY_ROOT / "AGENTS.md").read_text(
            encoding="utf-8"
        )
        product_marker = (
            REPOSITORY_ROOT / "harness/template/AGENTS.md.jinja"
        ).read_text(encoding="utf-8")
        source_gate = source_manifest.index("Run the source `hive-usage-guard`")
        source_discipline = source_manifest.index(
            "After that gate allows work and before editing anything"
        )
        source_loading = source_manifest.index(
            ".agents/directives/00-editing-discipline.md",
            source_manifest.index("## Mandatory Directive Loading"),
        )
        self.assertLess(source_gate, source_discipline)
        self.assertLess(source_discipline, source_loading)
        product_discipline = product_marker.index("Before editing anything")
        product_gate = product_marker.index(
            "Immediately before each new automatic dispatch"
        )
        self.assertLess(product_discipline, product_gate)
        for surface in (source_manifest, product_marker):
            self.assertIn("highest-priority editing discipline", surface)
            self.assertIn("never compact, summarize, omit, or substitute", surface)
        self.assertIn("literal `# CLAUDE.md` heading is original text", product_marker)
        self.assertIn("Codex, Claude, and Gemini Antigravity", product_marker)

    def test_source_and_consumer_human_documentation_style_contract(self) -> None:
        source_behavior = (
            REPOSITORY_ROOT / ".agents/directives/01-behavior.md"
        ).read_text(encoding="utf-8")
        normalized_source_behavior = " ".join(source_behavior.split())
        source_documentation = (
            REPOSITORY_ROOT / ".agents/directives/04-documentation-state.md"
        ).read_text(encoding="utf-8")
        source_directive = (
            REPOSITORY_ROOT
            / ".agents/directives/08-human-documentation-style.md"
        ).read_text(encoding="utf-8")
        source_manifest = (REPOSITORY_ROOT / "AGENTS.md").read_text(
            encoding="utf-8"
        )
        template = (
            REPOSITORY_ROOT / "harness/template/AGENTS.md.jinja"
        ).read_text(encoding="utf-8")
        renderer = (
            REPOSITORY_ROOT / "crates/hive-render/src/lib.rs"
        ).read_text(encoding="utf-8")
        guidance = (
            REPOSITORY_ROOT / "docs/guidance-schema.md"
        ).read_text(encoding="utf-8")
        user_guidance_renderer = (
            REPOSITORY_ROOT / "crates/hive-cli/src/user_install.rs"
        ).read_text(encoding="utf-8")
        shipped_rule = (
            "Write human-readable project documents in concise Korean unless "
            "the user explicitly requests another language."
        )
        source_response_language_rule = (
            "Respond to the maintainer in Korean unless the maintainer explicitly "
            "requests another language for the current response."
        )
        selected_language_rule = (
            "Use the selected interface language `{{ interface_language }}` for "
            "every question and response unless the user explicitly requests another "
            "language for the current response."
        )
        message_language_non_override_rule = (
            "A message written in another language does not by itself change this "
            "preference."
        )
        automatic_handoff_rule = (
            "Before presenting pending actions or a user handoff, complete every "
            "safe, in-scope, automatable action"
        )
        persisted_plan_rule = (
            "Unless the user explicitly opts out for the current request, write every "
            "plan to an appropriate project Markdown file"
        )
        explicit_result_scope_rule = (
            "failed, skipped, deferred, unverified, or unsupported item"
        )
        simple_explanation_rule = "Explain in simple terms by default."
        precise_example_rule = (
            "do not force irrelevant examples or weaken technical precision"
        )

        self.assertIn("Do not insert replaceable English general nouns", source_behavior)
        self.assertIn("write the full passage in English", source_behavior)
        self.assertIn("Keep each passage in one base language", source_directive)
        self.assertIn("Replace ordinary English nouns", source_directive)
        self.assertIn("examples, not an exhaustive allowlist", source_directive)
        self.assertIn("`~한다`", source_directive)
        self.assertIn(
            "`Aigent Hive는 provider-neutral 로컬 agent harness다.`",
            source_directive,
        )
        self.assertIn(
            "`Aigent Hive: provider-neutral 로컬 agent harness`",
            source_directive,
        )
        self.assertIn("Do not mechanically replace", source_directive)
        self.assertIn("Blockquote syntax alone never", source_directive)
        self.assertIn(
            ".agents/directives/08-human-documentation-style.md",
            source_manifest,
        )
        self.assertIn(shipped_rule, template)
        self.assertIn(shipped_rule, renderer)
        self.assertIn(source_response_language_rule, source_manifest)
        self.assertIn(source_response_language_rule, normalized_source_behavior)
        self.assertIn(simple_explanation_rule, source_behavior)
        self.assertIn(precise_example_rule, source_behavior)
        self.assertIn(selected_language_rule, template)
        self.assertIn(message_language_non_override_rule, template)
        self.assertIn(simple_explanation_rule, template)
        self.assertIn(precise_example_rule, template)
        self.assertIn(simple_explanation_rule, renderer)
        self.assertIn(precise_example_rule, renderer)
        self.assertIn(message_language_non_override_rule, renderer)
        self.assertIn("Before presenting a to-do list", source_behavior)
        self.assertIn("present only genuinely user-owned actions", source_behavior)
        self.assertIn(automatic_handoff_rule, template)
        self.assertIn(automatic_handoff_rule, renderer)
        self.assertIn("Never mirror a persisted plan one-for-one", source_documentation)
        self.assertIn(persisted_plan_rule, template)
        self.assertIn(persisted_plan_rule, renderer)
        self.assertIn(explicit_result_scope_rule, source_behavior)
        self.assertIn(explicit_result_scope_rule, template)
        self.assertIn(explicit_result_scope_rule, renderer)
        self.assertIn("남은 작업 목록·인계 전", guidance)
        self.assertIn("자동 처리 불가 이유", guidance)
        self.assertIn("Session의 persisted 계획 전문 일대일 복제 금지", guidance)
        self.assertIn("통과·실패·건너뜀·연기·미검증·미지원 결과", guidance)
        self.assertIn("해석에 필요한 한정어를 간결함을 이유로 생략 금지", guidance)
        self.assertIn("기본 설명은 쉬운 말과 직접적인 표현 우선", guidance)
        self.assertIn("관련 없는 예시 강제와 기술적 정확성 약화 금지", guidance)
        self.assertIn("finish every safe, in-scope, automatable task", user_guidance_renderer)
        self.assertIn("남은 작업 제시 전", user_guidance_renderer)
        self.assertIn("write every plan to an appropriate project Markdown file", user_guidance_renderer)
        self.assertIn("저장한 계획 전문을 session에 일대일 복제하지 않고", user_guidance_renderer)
        self.assertIn(explicit_result_scope_rule, user_guidance_renderer)
        self.assertIn("통과·실패·건너뜀·연기·미검증·미지원", user_guidance_renderer)
        self.assertIn(simple_explanation_rule, user_guidance_renderer)
        self.assertIn(precise_example_rule, user_guidance_renderer)
        self.assertIn("기본 설명은 쉬운 말로 작성", user_guidance_renderer)
        self.assertIn(
            "관련 없는 예시 강제 또는 기술적 정확성 약화 금지",
            user_guidance_renderer,
        )
        self.assertIn("대체 가능한 일반 영어 단어의 한영 혼용 금지", guidance)
        self.assertIn(
            "대체 가능한 일반 영어 단어의 한영 혼용 금지",
            user_guidance_renderer,
        )
        self.assertIn(
            "use English for every question and response unless the user explicitly "
            "requests another language for the current response",
            user_guidance_renderer,
        )
        self.assertIn(message_language_non_override_rule, user_guidance_renderer)
        self.assertIn(
            "다른 언어로 작성된 메시지만으로 이 선호를 변경하지 않음",
            user_guidance_renderer,
        )
        exact_pairs = (
            (
                "Aigent Hive는 provider-neutral 로컬 agent harness다.",
                "Aigent Hive: provider-neutral 로컬 agent harness",
            ),
            ("Product version은 0.7.0이다.", "Product version: 0.7.0"),
            ("Release 계약이 구현됐다.", "Release 계약 구현 완료"),
            (
                "API key를 요청하거나 저장하지 않는다.",
                "API key 요청·저장 없음",
            ),
            ("이 기능을 사용합니다.", "기능 사용"),
            ("다음 단계에서 검증해요.", "다음 단계: 검증"),
            ("검증이 필요합니다.", "검증 필요"),
            ("업데이트가 완료되었습니다.", "업데이트 완료"),
            ("Release 계약이 구현됐음.", "Release 계약 구현 완료"),
            (
                "API key를 요청하거나 저장하지 않음.",
                "API key 요청·저장 없음",
            ),
            ("Status는 INDETERMINATE다.", "Status: INDETERMINATE"),
            ("문서를 읽음.", "문서 확인"),
            ("작업이 끝남.", "작업 완료"),
            ("연결이 닫힘.", "연결 종료"),
            ("설정 값을 가짐.", "설정 값 보유"),
            ("정책을 따름.", "정책 준수"),
            ("compile됨.", "compile 완료"),
            ("검증할 수 있음.", "검증 가능"),
            ("검증할 수 없음.", "검증 불가"),
            ("문서를 보여 줘.", "문서 확인 요청"),
            ("기능을 사용해.", "기능 사용 요청"),
        )
        for exact_example in {
            example for pair in exact_pairs for example in pair
        }:
            self.assertIn(exact_example, source_directive)
            self.assertIn(exact_example, template)
            self.assertIn(exact_example, renderer)
            self.assertIn(exact_example, guidance)
        self.assertIn("## 사람용 문서 스타일", guidance)
        self.assertIn("예시는 제한 목록 아님", guidance)
        self.assertIn("Blockquote 표시는 exact quote 증거 아님", guidance)
        self.assertIn("`Release 계약이 구현됐음.`", guidance)
        self.assertIn("possibility clauses", template)
        self.assertIn("conversational imperative endings", template)
        self.assertIn("Conversational imperative", source_directive)

    def test_default_on_wiki_autocaptures_reviewed_task_facts(self) -> None:
        source_manifest = (REPOSITORY_ROOT / "AGENTS.md").read_text(
            encoding="utf-8"
        )
        source_directive = (
            REPOSITORY_ROOT / ".agents/directives/04-documentation-state.md"
        ).read_text(encoding="utf-8")
        project_directive = (
            REPOSITORY_ROOT / "harness/directives/01-project-knowledge.md"
        ).read_text(encoding="utf-8")
        template_directive = (
            REPOSITORY_ROOT
            / "harness/template/.agents/directives/01-project-knowledge.md"
        ).read_text(encoding="utf-8")
        template_marker = (
            REPOSITORY_ROOT / "harness/template/AGENTS.md.jinja"
        ).read_text(encoding="utf-8")
        capture_skill = (
            REPOSITORY_ROOT / "harness/skills/hive-knowledge-capture/SKILL.md"
        ).read_text(encoding="utf-8")

        self.assertEqual(project_directive, template_directive)
        for surface in (
            source_manifest,
            source_directive,
            project_directive,
            template_marker,
            capture_skill,
        ):
            self.assertIn("agent-reviewed", surface.lower())
            self.assertIn("task fact", surface.lower().replace("-", " "))
            self.assertIn("raw transcript", surface.lower())
        for field in ("outcome", "tool", "criteria", "originating request"):
            self.assertIn(field, project_directive.lower())
        self.assertIn('"enabled" if wiki_enabled else "disabled"', template_marker)
        self.assertIn("Do not capture when Wiki is disabled", capture_skill)
        self.assertNotIn("automatic memory ingestion", capture_skill)

    def test_consumer_turn_gate_uses_enforcement_and_semantic_intent(self) -> None:
        template = (
            REPOSITORY_ROOT / "harness/template/AGENTS.md.jinja"
        ).read_text(encoding="utf-8")
        renderer = (
            REPOSITORY_ROOT / "crates/hive-render/src/lib.rs"
        ).read_text(encoding="utf-8")
        canonical_skill = (
            REPOSITORY_ROOT / "harness/skills/hive-usage-guard/SKILL.md"
        ).read_text(encoding="utf-8")
        projected_skill = (
            REPOSITORY_ROOT
            / "harness/template/.agents"
            / "skills/hive-usage-guard/SKILL.md"
        ).read_text(encoding="utf-8")
        guidance = (
            REPOSITORY_ROOT / "docs/guidance-schema.md"
        ).read_text(encoding="utf-8")
        readme = (REPOSITORY_ROOT / "README.md").read_text(encoding="utf-8")
        korean_readme = (
            REPOSITORY_ROOT / "docs/readme/README.ko.md"
        ).read_text(encoding="utf-8")

        self.assertEqual(canonical_skill, projected_skill)
        for surface in (
            template,
            renderer,
            canonical_skill,
            guidance,
            readme,
            korean_readme,
        ):
            self.assertIn("hive usage enforce", surface)
            self.assertIn("hive run resume --dispatch-intent automatic", surface)
        for surface in (template, renderer, canonical_skill):
            normalized_surface = surface.lower().replace(
                "current-session", "session"
            )
            self.assertIn("bare continue", surface.lower())
            self.assertIn("Immediately before each new automatic dispatch", surface)
            self.assertIn("ordinary", surface)
            self.assertIn("manual", surface)
            self.assertIn("non-dispatch", surface)
            self.assertIn("exit `0`", normalized_surface)
            self.assertIn("preflight", surface)
            self.assertIn("never authorizes dispatch", surface)
            self.assertIn("enforced=true", surface)
            self.assertIn("outcome=authorized", surface)
            self.assertIn("authorization ID", surface)
            self.assertIn("exactly one dispatch brief", surface)
            self.assertIn("current halt marker takes priority", normalized_surface)
            self.assertIn("session disable", normalized_surface)
            self.assertIn("does not authorize dispatch", surface)
            self.assertIn("non-codex", normalized_surface)
            self.assertIn("fails closed", normalized_surface)
            self.assertNotIn("At every turn boundary", surface)
        self.assertIn("finite phrase list", template)
        self.assertIn("finite phrase list", renderer)
        self.assertIn("illustrative rather than a finite phrase", canonical_skill)
        for surface in (template, renderer, canonical_skill):
            self.assertIn("auxiliary evidence", surface)
        for surface in (guidance, korean_readme):
            normalized = " ".join(surface.split())
            self.assertIn("cancellation 결과는 보조 evidence", normalized)
            self.assertIn("durable goal/task 상태 대체 불가", normalized)
        normalized_readme = " ".join(readme.split()).lower()
        self.assertIn("cancellation is auxiliary evidence", normalized_readme)
        self.assertIn("never replaces durable goal/task state", normalized_readme)
        self.assertIn("start a watcher", canonical_skill)
        self.assertIn("Never install a fallback hook", canonical_skill)
        usage_control = (
            REPOSITORY_ROOT / "crates/hive-cli/src/usage_control.rs"
        ).read_text(encoding="utf-8")
        self.assertIn('"authorizes_dispatch": false', usage_control)
        self.assertIn('"scope": "automatic-dispatch-preflight"', usage_control)

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

    def test_current_plugin_skill_bodies_match_canonical_sources(self) -> None:
        plugin_root = REPOSITORY_ROOT / "harness/plugins/aigent-hive/skills"
        for name in sorted(IMPLEMENTED_SKILLS):
            with self.subTest(name=name):
                self.assertEqual(
                    (plugin_root / name / "SKILL.md").read_bytes(),
                    (SKILL_ROOT / name / "SKILL.md").read_bytes(),
                )

    def test_codex_skill_metadata_has_one_bounded_implicit_projection(
        self,
    ) -> None:
        plugin_root = (
            REPOSITORY_ROOT / "harness/plugins/aigent-hive/skills"
        )
        compatibility_root = (
            REPOSITORY_ROOT / "harness/template/.agents/skills"
        )
        implicit_description_chars = 0

        for name in sorted(IMPLEMENTED_SKILLS):
            with self.subTest(name=name):
                source = SKILL_ROOT / name
                metadata_path = source / "agents/openai.yaml"
                metadata = read_yaml(metadata_path)
                interface = metadata.get("interface")
                policy = metadata.get("policy")
                self.assertIsInstance(interface, dict)
                self.assertIsInstance(policy, dict)
                self.assertEqual(
                    policy.get("allow_implicit_invocation"),
                    name in IMPLICIT_PLUGIN_SKILLS,
                )
                self.assertIn(f"${name}", interface.get("default_prompt", ""))
                self.assertEqual(
                    (plugin_root / name / "agents/openai.yaml").read_bytes(),
                    metadata_path.read_bytes(),
                )

                if name == "setup-hive":
                    self.assertFalse((compatibility_root / name).exists())
                    continue

                compatibility = read_yaml(
                    compatibility_root / name / "agents/openai.yaml"
                )
                compatibility_policy = compatibility.get("policy")
                self.assertIsInstance(compatibility_policy, dict)
                self.assertFalse(
                    compatibility_policy.get("allow_implicit_invocation")
                )
                compatibility_interface = compatibility.get("interface")
                self.assertIsInstance(compatibility_interface, dict)
                self.assertIn(
                    f"${name}",
                    compatibility_interface.get("default_prompt", ""),
                )

                if name in IMPLICIT_PLUGIN_SKILLS:
                    frontmatter, _ = skill_frontmatter(
                        source / "SKILL.md"
                    )
                    description = str(frontmatter["description"])
                    self.assertLessEqual(len(description), 240)
                    implicit_description_chars += len(description)

        self.assertLessEqual(
            implicit_description_chars,
            IMPLICIT_DESCRIPTION_BUDGET,
        )

        source_skill_root = REPOSITORY_ROOT / ".agents/skills"
        overlapping_source_skills = {
            path.name
            for path in source_skill_root.iterdir()
            if path.is_dir() and path.name in IMPLEMENTED_SKILLS
        }
        self.assertEqual(
            overlapping_source_skills,
            {
                "ai-slop-cleaner",
                "auto-setup-harness",
                "best-practice-research",
                "hive-knowledge-capture",
                "hive-knowledge-query",
                "hive-knowledge-scan",
                "hive-loop-engineering",
                "hive-prompt-refine",
                "hive-simple-question",
                "hive-usage-guard",
                "hive-wiki",
            },
        )
        for name in overlapping_source_skills:
            source_metadata = read_yaml(
                source_skill_root / name / "agents/openai.yaml"
            )
            source_policy = source_metadata.get("policy")
            self.assertIsInstance(source_policy, dict)
            self.assertFalse(
                source_policy.get("allow_implicit_invocation")
            )

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
            / "harness/template/.agents/skills"
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
