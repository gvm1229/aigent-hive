#!/usr/bin/env python3
"""Black-box Phase 1 setup, consent, ownership, and migration conformance."""

from __future__ import annotations

import copy
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
FIXTURE_ROOT = REPOSITORY_ROOT / "tests/fixtures/phase1"
ACTION_RESULT_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/action-result.schema.json").read_text(encoding="utf-8")
)
HIVE_MARKER_START = b"<!-- AIGENT-HIVE:START -->"
HIVE_MARKER_END = b"<!-- AIGENT-HIVE:END -->"


def read_yaml(path: Path) -> dict[str, object]:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected YAML object: {path}")
    return value


def write_answers(path: Path, answers: dict[str, object]) -> None:
    path.write_text(
        yaml.safe_dump(answers, allow_unicode=True, sort_keys=False),
        encoding="utf-8",
    )


def snapshot_tree(root: Path) -> dict[str, tuple[str, bytes | str]]:
    snapshot: dict[str, tuple[str, bytes | str]] = {}
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            snapshot[relative] = ("symlink", os.readlink(path))
        elif path.is_file():
            snapshot[relative] = ("file", path.read_bytes())
        elif path.is_dir():
            snapshot[relative] = ("directory", "")
    return snapshot


def split_hive_marker(content: bytes) -> tuple[bytes, bytes]:
    start = content.find(HIVE_MARKER_START)
    if start < 0:
        raise AssertionError("Hive marker start is missing")
    end = content.find(HIVE_MARKER_END, start + len(HIVE_MARKER_START))
    if end < 0:
        raise AssertionError("Hive marker end is missing")
    end += len(HIVE_MARKER_END)
    return content[:start], content[end:]


class Phase1SetupConformance(unittest.TestCase):
    """Exercise the compiled CLI against isolated synthetic consumer projects."""

    @classmethod
    def setUpClass(cls) -> None:
        configured_binary = os.environ.get("HIVE_BIN")
        if configured_binary:
            cls.hive_binary = Path(configured_binary).resolve()
            return
        subprocess.run(
            ["cargo", "build", "--quiet", "--bin", "hive"],
            cwd=REPOSITORY_ROOT,
            check=True,
        )
        executable = "hive.exe" if os.name == "nt" else "hive"
        cls.hive_binary = REPOSITORY_ROOT / "target/debug" / executable

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory(
            prefix="aigent-hive-phase1-"
        )
        self.work_root = Path(self.temporary_directory.name)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def invoke_setup(
        self,
        target: Path,
        *,
        answers: Path | None = None,
        capabilities: str = "capabilities-codex-omx.json",
        mode: str = "--apply",
        reconfigure_roles: tuple[str, ...] = (),
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        command = [
            str(self.hive_binary),
            "setup",
            "--target",
            str(target),
            "--answers",
            str(answers or FIXTURE_ROOT / "answers-base.yml"),
            "--capabilities",
            str(FIXTURE_ROOT / capabilities),
            mode,
        ]
        for role_id in reconfigure_roles:
            command.extend(["--reconfigure-role", role_id])
        command.extend(["--output", "json"])
        process = subprocess.run(
            command,
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"stdout must contain exactly one JSON object: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        self.assertIsInstance(result, dict)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"])
        return process, result

    def assert_owner_evidence(
        self, result: dict[str, object], expected_owner: str
    ) -> None:
        evidence = result["evidence"]
        self.assertIsInstance(evidence, list)
        locators = {
            item["locator"]
            for item in evidence
            if isinstance(item, dict) and item.get("kind") == "report"
        }
        self.assertIn(f"orchestration-owner:{expected_owner}", locators)

    def assert_no_hook_artifacts(self, target: Path) -> None:
        self.assertFalse((target / ".hive/config/approved-hooks.yml").exists())
        self.assertFalse((target / ".hive/hooks").exists())

    def copied_answers(
        self, fixture_name: str = "answers-base.yml"
    ) -> tuple[Path, dict[str, object]]:
        answers = read_yaml(FIXTURE_ROOT / fixture_name)
        destination = self.work_root / fixture_name
        write_answers(destination, answers)
        return destination, answers

    def assert_hook_consent_tamper_blocks(
        self,
        field: str,
        value: object,
        expected_exit: int,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers("answers-partial-hooks.yml")
        hooks = answers["approved_fallback_hooks"]
        self.assertIsInstance(hooks, list)
        hooks[0][field] = value
        write_answers(answer_path, answers)

        process, result = self.invoke_setup(
            target,
            answers=answer_path,
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, expected_exit)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), {})

    def test_codex_uses_compatible_omx_owner_automatically(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(target)

        self.assertEqual(process.returncode, 0)
        self.assert_owner_evidence(result, "omx")

    def test_claude_uses_compatible_omc_owner_automatically(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers()
        answers["primary_host"] = "claude"
        write_answers(answer_path, answers)

        process, result = self.invoke_setup(
            target,
            answers=answer_path,
            capabilities="capabilities-claude-omc.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner_evidence(result, "omc")

    def test_absent_external_runtime_uses_host_native_owner(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner_evidence(result, "host-native")

    def test_available_external_runtime_installs_no_fallback_hooks(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, _ = self.invoke_setup(target)

        self.assertEqual(process.returncode, 0)
        self.assert_no_hook_artifacts(target)

    def test_incompatible_external_runtime_installs_no_fallback_hooks(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            capabilities="capabilities-incompatible.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner_evidence(result, "host-native")
        self.assert_no_hook_artifacts(target)

    def test_unknown_external_runtime_installs_no_fallback_hooks(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            capabilities="capabilities-unknown.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner_evidence(result, "host-native")
        self.assert_no_hook_artifacts(target)

    def test_incompatible_external_runtime_rejects_injected_hook_approval(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-incompatible.json",
        )

        self.assertEqual(process.returncode, 3)
        self.assert_no_hook_artifacts(target)

    def test_unknown_external_runtime_rejects_injected_hook_approval(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-unknown.json",
        )

        self.assertEqual(process.returncode, 3)
        self.assert_no_hook_artifacts(target)

    def test_available_external_runtime_rejects_injected_hook_approval(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
        )

        self.assertEqual(process.returncode, 3)
        self.assert_no_hook_artifacts(target)

    def test_absent_external_runtime_allows_declining_all_hooks(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, _ = self.invoke_setup(
            target,
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_no_hook_artifacts(target)

    def test_absent_external_runtime_projects_only_approved_hooks(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, 0)
        approvals = read_yaml(target / ".hive/config/approved-hooks.yml")
        hooks = approvals["hooks"]
        self.assertEqual(
            {hook["capability"] for hook in hooks},
            {"protect-hive-owned-state", "checkpoint-reminder"},
        )
        self.assertTrue(
            (target / ".hive/hooks/protect-hive-owned-state").is_file()
        )
        self.assertTrue((target / ".hive/hooks/checkpoint-reminder").is_file())
        self.assertFalse((target / ".hive/hooks/update-integrity-guard").exists())
        self.assertFalse((target / ".hive/hooks/derived-state-invalidation").exists())

    def test_hook_consent_tamper_blocks_before_staging(self) -> None:
        self.assert_hook_consent_tamper_blocks(
            "command",
            "hive hook --capability protect-hive-owned-state "
            "--event PreToolUse --output human",
            2,
        )

    def test_hook_event_tamper_blocks_before_staging(self) -> None:
        self.assert_hook_consent_tamper_blocks(
            "event",
            "PostToolUse",
            2,
        )

    def test_hook_capability_tamper_blocks_before_staging(self) -> None:
        self.assert_hook_consent_tamper_blocks(
            "capability",
            "update-integrity-guard",
            2,
        )

    def test_hook_content_digest_tamper_blocks_before_staging(self) -> None:
        self.assert_hook_consent_tamper_blocks(
            "content_digest",
            f"sha256:{'9' * 64}",
            3,
        )

    def test_hook_path_tamper_blocks_before_staging(self) -> None:
        self.assert_hook_consent_tamper_blocks(
            "path",
            ".hive/hooks/renamed-protect-hook",
            2,
        )

    def test_stop_hook_returns_neutral_allow_without_continuation(self) -> None:
        process = subprocess.run(
            [
                str(self.hive_binary),
                "hook",
                "--capability",
                "checkpoint-reminder",
                "--event",
                "Stop",
                "--input",
                str(FIXTURE_ROOT / "stop-input.json"),
                "--output",
                "json",
            ],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        output = json.loads(process.stdout)
        self.assertEqual(output.get("decision"), "allow")
        forbidden = {"block", "continue", "continuation", "prompt", "reason"}
        self.assertTrue(forbidden.isdisjoint(output))

    def test_optional_skill_capability_subset_violation_is_rejected(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers("answers-approved-skill.yml")
        skills = answers["approved_optional_skills"]
        self.assertIsInstance(skills, list)
        skill = skills[0]
        skill["approved_capabilities"] = ["filesystem-read", "shell"]
        skill["consent_digest"] = (
            "sha256:c211b2b97ab2c201f5dbf486bd80eeb8"
            "fabedb3f0a9bcd1f4d5f93f4c79d0260"
        )
        write_answers(answer_path, answers)

        process, _ = self.invoke_setup(target, answers=answer_path)

        self.assertEqual(process.returncode, 2)
        self.assertEqual(snapshot_tree(target), {})

    def test_optional_skill_consent_tamper_blocks_activation(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers("answers-approved-skill.yml")
        skills = answers["approved_optional_skills"]
        self.assertIsInstance(skills, list)
        skills[0]["source"] = "https://example.invalid/tampered"
        write_answers(answer_path, answers)

        process, _ = self.invoke_setup(target, answers=answer_path)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(snapshot_tree(target), {})

    def test_same_inputs_render_byte_identical_trees(self) -> None:
        first_target = self.work_root / "first"
        second_target = self.work_root / "second"
        first_target.mkdir()
        second_target.mkdir()

        first_process, _ = self.invoke_setup(first_target)
        second_process, _ = self.invoke_setup(second_target)

        self.assertEqual(first_process.returncode, 0)
        self.assertEqual(second_process.returncode, 0)
        self.assertEqual(snapshot_tree(first_target), snapshot_tree(second_target))

    def test_source_workspace_guard_prevents_every_write(self) -> None:
        target = self.work_root / "source"
        target.mkdir()
        (target / "hive-source.json").write_text("{}\n", encoding="utf-8")
        sentinel = target / "user.txt"
        sentinel.write_bytes(b"user bytes\n")
        before = snapshot_tree(target)

        process, _ = self.invoke_setup(target)

        self.assertEqual(process.returncode, 2)
        self.assertEqual(snapshot_tree(target), before)

    def test_hook_path_traversal_is_rejected_before_external_write(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers("answers-partial-hooks.yml")
        hooks = answers["approved_fallback_hooks"]
        self.assertIsInstance(hooks, list)
        hook = hooks[0]
        hook["path"] = "../escaped-hook"
        write_answers(answer_path, answers)

        process, _ = self.invoke_setup(
            target,
            answers=answer_path,
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, 2)
        self.assertFalse((self.work_root / "escaped-hook").exists())

    def test_symlink_escape_is_rejected_without_writing_link_target(self) -> None:
        target = self.work_root / "consumer"
        outside = self.work_root / "outside"
        target.mkdir()
        outside.mkdir()
        (target / ".hive").symlink_to(outside, target_is_directory=True)

        process, _ = self.invoke_setup(target)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(snapshot_tree(outside), {})

    def test_shared_marker_merge_preserves_all_non_hive_bytes(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        agents_path = target / "AGENTS.md"
        shutil.copyfile(FIXTURE_ROOT / "shared-agents.md", agents_path)
        original_prefix, original_suffix = split_hive_marker(agents_path.read_bytes())

        process, _ = self.invoke_setup(target)

        self.assertEqual(process.returncode, 0)
        rendered_prefix, rendered_suffix = split_hive_marker(agents_path.read_bytes())
        self.assertEqual(rendered_prefix, original_prefix)
        self.assertEqual(rendered_suffix, original_suffix)

    def test_nested_shared_markers_conflict_without_changing_user_file(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        agents_path = target / "AGENTS.md"
        shutil.copyfile(FIXTURE_ROOT / "conflicting-agents.md", agents_path)
        original = agents_path.read_bytes()

        process, _ = self.invoke_setup(target)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(agents_path.read_bytes(), original)

    def test_role_materialization_is_byte_identical_on_second_setup(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        first_process, _ = self.invoke_setup(target)
        role_path = target / ".hive/team/roles/reviewer.md"
        first_role = role_path.read_bytes()
        second_process, _ = self.invoke_setup(target)

        self.assertEqual(first_process.returncode, 0)
        self.assertEqual(second_process.returncode, 0)
        self.assertEqual(role_path.read_bytes(), first_role)

    def test_unapproved_role_definition_drift_preserves_existing_role(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(target)
        self.assertEqual(first_process.returncode, 0)
        role_path = target / ".hive/team/roles/reviewer.md"
        original = role_path.read_bytes()
        answer_path, answers = self.copied_answers()
        roles = answers["persistent_roles"]
        self.assertIsInstance(roles, list)
        roles[0]["display_name"] = "Senior Reviewer"
        write_answers(answer_path, answers)

        process, _ = self.invoke_setup(target, answers=answer_path)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(role_path.read_bytes(), original)

    def test_approved_role_reconfigure_preserves_runtime_state_and_body(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(target)
        self.assertEqual(first_process.returncode, 0)
        role_path = target / ".hive/team/roles/reviewer.md"
        role_text = role_path.read_text(encoding="utf-8")
        frontmatter_text, _ = role_text.removeprefix("---\n").split("\n---\n", 1)
        profile = json.loads(frontmatter_text)
        profile["current_assignment"] = "audit Phase 1"
        profile["handoff_path"] = ".hive/runs/phase1/HANDOFF.md"
        custom_body = (
            "# Reviewer\n\nUser-maintained role notes.\n\n"
            "## Current assignment\n\nDo not overwrite this body.\n"
        )
        role_path.write_text(
            "---\n"
            + json.dumps(
                profile,
                ensure_ascii=False,
                separators=(",", ":"),
                sort_keys=True,
            )
            + "\n---\n"
            + custom_body,
            encoding="utf-8",
        )
        answer_path, answers = self.copied_answers()
        roles = answers["persistent_roles"]
        self.assertIsInstance(roles, list)
        roles[0]["display_name"] = "Senior Reviewer"
        write_answers(answer_path, answers)

        process, _ = self.invoke_setup(
            target,
            answers=answer_path,
            reconfigure_roles=("reviewer",),
        )

        self.assertEqual(process.returncode, 0)
        updated_text = role_path.read_text(encoding="utf-8")
        updated_frontmatter, updated_body = updated_text.removeprefix("---\n").split(
            "\n---\n", 1
        )
        updated_profile = json.loads(updated_frontmatter)
        self.assertEqual(updated_profile["display_name"], "Senior Reviewer")
        self.assertEqual(updated_profile["current_assignment"], "audit Phase 1")
        self.assertEqual(
            updated_profile["handoff_path"],
            ".hive/runs/phase1/HANDOFF.md",
        )
        self.assertEqual(updated_body, custom_body)

    def test_legacy_orchestration_answer_is_removed_during_migration(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-legacy-v1.yml",
        )

        self.assertEqual(process.returncode, 0)
        installed_answers = read_yaml(target / ".hive/setup-answers.yml")
        self.assertNotIn("orchestration_layer", installed_answers)

    def test_validate_reports_verification_failure_for_tampered_skill_ledger(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-approved-skill.yml",
        )
        self.assertEqual(first_process.returncode, 0)
        ledger_path = target / ".hive/config/approved-skills.yml"
        ledger = read_yaml(ledger_path)
        skills = ledger["skills"]
        self.assertIsInstance(skills, list)
        skills[0]["source"] = "https://example.invalid/tampered"
        write_answers(ledger_path, ledger)
        tampered = ledger_path.read_bytes()

        process, result = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-approved-skill.yml",
            mode="--validate",
        )

        self.assertEqual(process.returncode, 5)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(ledger_path.read_bytes(), tampered)

    def test_conflicting_modes_return_schema_valid_input_error(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        command = [
            str(self.hive_binary),
            "setup",
            "--target",
            str(target),
            "--answers",
            str(FIXTURE_ROOT / "answers-base.yml"),
            "--capabilities",
            str(FIXTURE_ROOT / "capabilities-codex-omx.json"),
            "--dry-run",
            "--apply",
            "--output",
            "json",
        ]

        process = subprocess.run(
            command,
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )

        result = json.loads(process.stdout)
        Draft202012Validator(ACTION_RESULT_SCHEMA).validate(result)
        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["exit_code"], 2)
        self.assertEqual(result["changed_paths"], [])


if __name__ == "__main__":
    unittest.main()
