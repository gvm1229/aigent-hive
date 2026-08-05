#!/usr/bin/env python3
"""Copier parity, role lifecycle, and setup-answer migration conformance."""

from __future__ import annotations

import copy
import difflib
import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator

from tests.conformance.phase1_support import (
    EXPECTED_ROOT,
    FIXTURE_ROOT,
    REPOSITORY_ROOT,
    Phase1CliTestCase,
    read_yaml,
    snapshot_tree,
    write_yaml,
)


def normalized_copier_tree(
    root: Path,
) -> dict[str, tuple[str, bytes | str]]:
    snapshot = snapshot_tree(root)
    for path in tuple(snapshot):
        if path == ".copier-answers.yml" or path == ".hive/config/project-base.json":
            snapshot.pop(path)
        elif path == ".hive/index" or path.startswith(".hive/index/"):
            snapshot.pop(path)
    return snapshot


class Phase1CopierParity(Phase1CliTestCase):
    def copier_template_source(self) -> Path:
        template = self.work_root / "copier-template"
        if not template.exists():
            def ignore_source_only_paths(
                directory: str,
                names: list[str],
            ) -> set[str]:
                directory_path = Path(directory)
                ignored = {name for name in names if name == "node_modules"}
                if directory_path == REPOSITORY_ROOT:
                    ignored.update({".claude", ".git", "target"})
                elif directory_path == REPOSITORY_ROOT / ".agents":
                    ignored.add("work")
                elif directory_path == REPOSITORY_ROOT / "harness/template/.claude/skills":
                    user_only_directory = directory_path / "setup-hive"
                    if user_only_directory.is_dir() and not any(
                        user_only_directory.iterdir()
                    ):
                        ignored.add("setup-hive")
                return ignored

            shutil.copytree(
                REPOSITORY_ROOT,
                template,
                ignore=ignore_source_only_paths,
            )
        return template

    def assert_copier_trees_equal(
        self,
        rust_target: Path,
        copier_target: Path,
    ) -> None:
        rust_tree = normalized_copier_tree(rust_target)
        copier_tree = normalized_copier_tree(copier_target)
        if rust_tree == copier_tree:
            return
        differences = []
        for path in sorted(rust_tree.keys() | copier_tree.keys()):
            rust_entry = rust_tree.get(path)
            copier_entry = copier_tree.get(path)
            if rust_entry == copier_entry:
                continue
            if rust_entry is None:
                differences.append(f"{path}: Copier only")
                continue
            if copier_entry is None:
                differences.append(f"{path}: Rust only")
                continue
            rust_kind, rust_content = rust_entry
            copier_kind, copier_content = copier_entry
            if isinstance(rust_content, bytes):
                rust_summary = hashlib.sha256(rust_content).hexdigest()
            else:
                rust_summary = repr(rust_content)
            if isinstance(copier_content, bytes):
                copier_summary = hashlib.sha256(copier_content).hexdigest()
            else:
                copier_summary = repr(copier_content)
            differences.append(
                f"{path}: Rust {rust_kind} {rust_summary}; "
                f"Copier {copier_kind} {copier_summary}"
            )
            if isinstance(rust_content, bytes) and isinstance(
                copier_content, bytes
            ):
                try:
                    rust_lines = rust_content.decode("utf-8").splitlines(
                        keepends=True
                    )
                    copier_lines = copier_content.decode("utf-8").splitlines(
                        keepends=True
                    )
                except UnicodeDecodeError:
                    continue
                differences.extend(
                    difflib.unified_diff(
                        rust_lines,
                        copier_lines,
                        fromfile=f"rust/{path}",
                        tofile=f"copier/{path}",
                    )
                )
        self.fail("Copier/Rust tree mismatch:\n" + "\n".join(differences))

    def assert_builtin_projection(
        self,
        target: Path,
        *,
        host: str,
    ) -> None:
        projection_roots = [".agents"]
        if host == "claude":
            projection_roots.append(".claude")
        active_ledger_path = target / ".hive/config/active-skills.yml"
        active_ledger = read_yaml(active_ledger_path)
        skills = active_ledger["skills"]
        self.assertIsInstance(skills, list)
        expected_names = [
            "ai-slop-cleaner",
            "auto-setup-harness",
            "best-practice-research",
            "hive-judge-package",
            "hive-knowledge-capture",
            "hive-knowledge-maintenance",
            "hive-knowledge-promote",
            "hive-knowledge-query",
            "hive-knowledge-scan",
            "hive-loop-engineering",
            "hive-migrate",
            "hive-project-upgrade",
            "hive-prompt-refine",
            "hive-role-handoff",
            "hive-run-checkpoint",
            "hive-run-resume",
            "hive-simple-question",
            "hive-update",
            "hive-usage-guard",
            "hive-wiki",
            "setup-harness",
        ]
        self.assertEqual([entry["name"] for entry in skills], expected_names)
        for projection_root in projection_roots:
            projected_skill_root = target / projection_root / "skills"
            self.assertEqual(
                {path.name for path in projected_skill_root.iterdir()},
                set(expected_names),
            )
        for entry in skills:
            self.assertIsInstance(entry, dict)
            name = entry["name"]
            source = REPOSITORY_ROOT / f"harness/skills/{name}/SKILL.md"
            with self.subTest(host=host, skill=name):
                self.assertEqual(
                    entry["content_digest"],
                    f"sha256:{hashlib.sha256(source.read_bytes()).hexdigest()}",
                )
                self.assertEqual(entry["source_type"], "built-in")
                self.assertIsNone(entry["consent_digest"])
                for projection_root in projection_roots:
                    projected = (
                        target
                        / projection_root
                        / f"skills/{name}/SKILL.md"
                    )
                    self.assertEqual(
                        projected.read_bytes(),
                        source.read_bytes(),
                    )
                    expected_entries = {"SKILL.md"}
                    if projection_root == ".agents":
                        expected_entries.add("agents")
                    self.assertEqual(
                        {path.name for path in projected.parent.iterdir()},
                        expected_entries,
                    )
        self.assertEqual((target / ".claude").exists(), host == "claude")
        ledger_bytes = active_ledger_path.read_bytes()
        self.assertNotIn(b".agents", ledger_bytes)
        self.assertNotIn(b".claude", ledger_bytes)

    def test_copier_and_rust_no_role_no_hook_static_trees_are_byte_equal(
        self,
    ) -> None:
        copier = os.environ.get("COPIER_BIN") or shutil.which("copier")
        self.assertIsNotNone(
            copier,
            "Copier 9.17.0 is required for the Phase 1 parity gate",
        )
        rust_target = self.work_root / "rust"
        copier_target = self.work_root / "copier"
        rust_target.mkdir()

        rust_process, _ = self.invoke_setup(
            rust_target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
            capabilities="capabilities-codex-omx.json",
        )
        copier_process = subprocess.run(
            [
                str(copier),
                "copy",
                "--trust",
                "--defaults",
                "--data-file",
                str(FIXTURE_ROOT / "copier-parity-data.yml"),
                str(self.copier_template_source()),
                str(copier_target),
            ],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )

        self.assertEqual(rust_process.returncode, 0)
        self.assertEqual(copier_process.returncode, 0, copier_process.stderr)
        self.assert_copier_trees_equal(rust_target, copier_target)

    def test_copier_and_rust_approved_hook_conditional_trees_are_byte_equal(
        self,
    ) -> None:
        copier = os.environ.get("COPIER_BIN") or shutil.which("copier")
        self.assertIsNotNone(
            copier,
            "Copier 9.17.0 is required for the Phase 1 parity gate",
        )
        rust_target = self.work_root / "rust-hooks"
        copier_target = self.work_root / "copier-hooks"
        rust_target.mkdir()

        rust_process, _ = self.invoke_setup(
            rust_target,
            answers=FIXTURE_ROOT / "answers-all-hooks.yml",
            capabilities="capabilities-codex-host-native-hooks.json",
        )
        copier_process = subprocess.run(
            [
                str(copier),
                "copy",
                "--trust",
                "--defaults",
                "--data-file",
                str(FIXTURE_ROOT / "copier-hooks-parity-data.yml"),
                str(self.copier_template_source()),
                str(copier_target),
            ],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )

        self.assertEqual(rust_process.returncode, 0, rust_process.stderr)
        self.assertEqual(copier_process.returncode, 0, copier_process.stderr)
        self.assert_copier_trees_equal(rust_target, copier_target)

    def test_copier_rejects_hooks_without_exact_supported_host_native_events(
        self,
    ) -> None:
        copier = os.environ.get("COPIER_BIN") or shutil.which("copier")
        self.assertIsNotNone(
            copier,
            "Copier 9.17.0 is required for the Phase 1 parity gate",
        )
        supported = json.loads(
            (FIXTURE_ROOT / "capabilities-codex-host-native-hooks.json").read_text(
                encoding="utf-8"
            )
        )
        absent = json.loads(
            (FIXTURE_ROOT / "capabilities-absent.json").read_text(encoding="utf-8")
        )
        external_owner = copy.deepcopy(supported)
        external_owner["resolved_owner"] = "omx"
        unsupported_event = copy.deepcopy(supported)
        unsupported_event["hook_events"]["PreToolUse"]["support"] = "unsupported"

        for resolution in (external_owner, unsupported_event):
            payload = {
                key: value
                for key, value in resolution.items()
                if key != "evidence_digest"
            }
            canonical = json.dumps(
                payload,
                ensure_ascii=False,
                separators=(",", ":"),
                sort_keys=True,
            ).encode("utf-8")
            resolution["evidence_digest"] = (
                "sha256:" + hashlib.sha256(canonical).hexdigest()
            )

        cases = {
            "absent": absent,
            "external-owner": external_owner,
            "unsupported-event": unsupported_event,
        }
        for name, resolution in cases.items():
            with self.subTest(name=name):
                copier_data = read_yaml(
                    FIXTURE_ROOT / "copier-hooks-parity-data.yml"
                )
                copier_data["capability_resolution"] = resolution
                data_path = self.work_root / f"copier-hooks-{name}.yml"
                write_yaml(data_path, copier_data)
                target = self.work_root / f"copier-hooks-{name}"
                process = subprocess.run(
                    [
                        str(copier),
                        "copy",
                        "--trust",
                        "--defaults",
                        "--data-file",
                        str(data_path),
                        str(self.copier_template_source()),
                        str(target),
                    ],
                    cwd=REPOSITORY_ROOT,
                    check=False,
                    text=True,
                    capture_output=True,
                )

                self.assertNotEqual(process.returncode, 0)
                self.assertIn("supported event", process.stderr)
                self.assertFalse(
                    (target / ".hive/config/approved-hooks.yml").exists()
                )
                self.assertFalse((target / ".hive/hooks").exists())

    def test_copier_and_rust_builtin_skill_trees_match_for_each_host(
        self,
    ) -> None:
        copier = os.environ.get("COPIER_BIN") or shutil.which("copier")
        self.assertIsNotNone(
            copier,
            "Copier 9.17.0 is required for the Phase 1 parity gate",
        )
        capability_fixture_by_host = {
            "codex": "capabilities-codex-omx.json",
            "claude": "capabilities-claude-omc.json",
            "antigravity": "capabilities-antigravity-absent.json",
        }

        for host, capability_fixture in capability_fixture_by_host.items():
            with self.subTest(host=host):
                answers = read_yaml(
                    FIXTURE_ROOT / "answers-no-role-no-hook.yml"
                )
                answers["primary_host"] = host
                answer_path = self.work_root / f"{host}-answers.yml"
                write_yaml(answer_path, answers)
                capability_resolution = json.loads(
                    (
                        FIXTURE_ROOT / capability_fixture
                    ).read_text(encoding="utf-8")
                )
                copier_data = {
                    key: value
                    for key, value in answers.items()
                    if key != "schema_version"
                }
                copier_data["capability_resolution"] = capability_resolution
                copier_data_path = self.work_root / f"{host}-copier-data.yml"
                write_yaml(copier_data_path, copier_data)

                rust_target = self.work_root / f"{host}-rust"
                copier_target = self.work_root / f"{host}-copier"
                rust_target.mkdir()
                rust_process, _ = self.invoke_setup(
                    rust_target,
                    answers=answer_path,
                    capabilities=capability_fixture,
                )
                copier_process = subprocess.run(
                    [
                        str(copier),
                        "copy",
                        "--trust",
                        "--defaults",
                        "--data-file",
                        str(copier_data_path),
                        str(self.copier_template_source()),
                        str(copier_target),
                    ],
                    cwd=REPOSITORY_ROOT,
                    check=False,
                    text=True,
                    capture_output=True,
                )

                self.assertEqual(
                    rust_process.returncode,
                    0,
                    rust_process.stderr,
                )
                self.assertEqual(
                    copier_process.returncode,
                    0,
                    copier_process.stderr,
                )
                self.assert_copier_trees_equal(rust_target, copier_target)
                self.assert_builtin_projection(rust_target, host=host)
                self.assert_builtin_projection(copier_target, host=host)


class Phase1RoleConformance(Phase1CliTestCase):
    def install_reviewer(self) -> tuple[Path, Path]:
        target = self.work_root / "consumer"
        target.mkdir()
        process, _ = self.invoke_setup(target)
        self.assertEqual(process.returncode, 0)
        return target, target / ".hive/team/roles/reviewer.md"

    def test_role_materialization_matches_exact_known_answer(self) -> None:
        _, role_path = self.install_reviewer()

        self.assertEqual(
            role_path.read_bytes(),
            (EXPECTED_ROOT / "reviewer-role.md").read_bytes(),
        )

    def test_role_frontmatter_satisfies_profile_schema(self) -> None:
        _, role_path = self.install_reviewer()
        role = role_path.read_text(encoding="utf-8")
        frontmatter, _ = role.removeprefix("---\n").split("\n---\n", 1)
        schema = json.loads(
            (REPOSITORY_ROOT / "schemas/role-profile.schema.json").read_text(
                encoding="utf-8"
            )
        )

        Draft202012Validator(schema).validate(json.loads(frontmatter))

    def test_role_frontmatter_keys_are_lexicographically_sorted(self) -> None:
        _, role_path = self.install_reviewer()
        role = role_path.read_text(encoding="utf-8")
        frontmatter, _ = role.removeprefix("---\n").split("\n---\n", 1)

        self.assertEqual(
            list(json.loads(frontmatter).keys()),
            [
                "allowed_capabilities",
                "context_paths",
                "current_assignment",
                "display_name",
                "handoff_path",
                "non_responsibilities",
                "responsibilities",
                "role_id",
                "schema_version",
                "verification_duties",
                "write_scope",
            ],
        )

    def test_role_initial_markdown_body_matches_contract(self) -> None:
        _, role_path = self.install_reviewer()
        role = role_path.read_text(encoding="utf-8")
        _, body = role.removeprefix("---\n").split("\n---\n", 1)

        self.assertEqual(
            body,
            "# Reviewer\n\n"
            "## Current assignment\n\n"
            "_Unassigned._\n\n"
            "## Handoff\n\n"
            "_No handoff yet._\n",
        )

    def assert_duplicate_role_ids_rejected(
        self,
        second_role_id: str,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers("answers-base.yml")
        roles = answers["persistent_roles"]
        self.assertIsInstance(roles, list)
        duplicate = copy.deepcopy(roles[0])
        duplicate["role_id"] = second_role_id
        roles.append(duplicate)
        write_yaml(answer_path, answers)
        before = snapshot_tree(self.work_root)

        process, result = self.invoke_setup(target, answers=answer_path)

        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.work_root), before)

    def test_exact_duplicate_role_ids_are_rejected_before_write(self) -> None:
        self.assert_duplicate_role_ids_rejected("reviewer")

    def test_casefold_colliding_role_ids_are_rejected_before_write(self) -> None:
        self.assert_duplicate_role_ids_rejected("Reviewer")

    def test_removing_role_seed_preserves_materialized_role_bytes(self) -> None:
        target, role_path = self.install_reviewer()
        original = role_path.read_bytes()

        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
        )

        self.assertEqual(process.returncode, 0)
        self.assertEqual(role_path.read_bytes(), original)

    def assert_definition_drift_conflicts_without_mutation(
        self,
        field: str,
        replacement: object,
    ) -> None:
        target, _ = self.install_reviewer()
        answer_path, answers = self.copied_answers("answers-base.yml")
        roles = answers["persistent_roles"]
        self.assertIsInstance(roles, list)
        roles[0][field] = replacement
        write_yaml(answer_path, answers)
        before = snapshot_tree(target)

        process, result = self.invoke_setup(target, answers=answer_path)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def test_malformed_active_role_causes_full_setup_rollback(self) -> None:
        target, role_path = self.install_reviewer()
        role_path.write_text("---\n{not-json}\n---\nuser body\n", encoding="utf-8")
        before = snapshot_tree(target)

        process, result = self.invoke_setup(target)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def test_cross_major_role_candidate_schema_failure_preserves_active_tree(
        self,
    ) -> None:
        target, role_path = self.install_reviewer()
        candidate = (FIXTURE_ROOT / "cross-major-role-malformed.md").read_bytes()
        candidate_text = candidate.decode("utf-8")
        frontmatter, _ = candidate_text.removeprefix("---\n").split("\n---\n", 1)
        candidate_profile = json.loads(frontmatter)
        self.assertEqual(candidate_profile["schema_version"], 999)
        role_path.write_bytes(candidate)
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            reconfigure_roles=("reviewer",),
        )

        self.assertEqual(process.returncode, 3)
        self.assertEqual(result["code"], "hive.setup-conflict")
        self.assertEqual(result["status"], "conflict")
        self.assertIn(
            "role profile violate the JSON Schema contract",
            result["message"],
        )
        self.assertNotIn("installed harness version parity failed", result["message"])
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)


ROLE_DEFINITION_DRIFT_CASES = {
    "display_name": "Changed Reviewer",
    "responsibilities": ["changed responsibility"],
    "non_responsibilities": ["changed non-responsibility"],
    "context_paths": ["src/**"],
    "allowed_capabilities": ["filesystem-read", "shell"],
    "write_scope": [".hive/knowledge/"],
    "verification_duties": ["changed verification duty"],
}


def make_definition_drift_test(field: str, replacement: object):
    def test(self: Phase1RoleConformance) -> None:
        self.assert_definition_drift_conflicts_without_mutation(
            field,
            replacement,
        )

    test.__name__ = f"test_{field}_definition_drift_conflicts_without_mutation"
    return test


for drift_field, drift_replacement in ROLE_DEFINITION_DRIFT_CASES.items():
    setattr(
        Phase1RoleConformance,
        f"test_{drift_field}_definition_drift_conflicts_without_mutation",
        make_definition_drift_test(drift_field, drift_replacement),
    )


class Phase1AnswerMigrationConformance(Phase1CliTestCase):
    def test_legacy_answers_migrate_losslessly_except_removed_owner_preference(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        legacy = read_yaml(FIXTURE_ROOT / "answers-legacy-lossless.yml")
        expected = {
            key: value
            for key, value in legacy.items()
            if key != "orchestration_layer"
        }
        expected["approved_fallback_hooks"] = []
        expected["project_identity"] = legacy["project_name"]
        expected["setup_mode"] = "expedited"
        expected["root_knowledge_promotion_categories"] = []
        expected["confidential_knowledge_categories"] = []
        expected["user_store_binding"] = ""

        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-legacy-lossless.yml",
            capabilities="capabilities-codex-omx.json",
        )

        self.assertEqual(process.returncode, 0)
        installed = read_yaml(target / ".hive/setup-answers.yml")
        self.assertEqual(installed, expected)

    def test_legacy_owner_preference_cannot_override_capability_resolution(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-legacy-lossless.yml",
            capabilities="capabilities-codex-omx.json",
        )

        self.assertEqual(process.returncode, 0)
        locators = {
            item["locator"]
            for item in result["evidence"]
            if isinstance(item, dict)
        }
        self.assertIn("orchestration-owner:omx", locators)
        self.assertNotIn("orchestration-owner:omc", locators)
