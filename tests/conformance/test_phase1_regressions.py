#!/usr/bin/env python3
"""Adversarial Phase 1 ownership, rollback, validation, and CLI regressions."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.phase1_support import (
    ACTION_RESULT_SCHEMA,
    FIXTURE_ROOT,
    Phase1CliTestCase,
    read_yaml,
    snapshot_tree,
    write_yaml,
)


PROTECTED_CANONICAL_PATHS = (
    ".hive/knowledge/Wiki/index.md",
    ".hive/knowledge/Wiki/log.md",
    ".hive/knowledge/Schema/schema.md",
    ".hive/knowledge/suppression.yml",
)


class Phase1ProtectedCanonicalData(Phase1CliTestCase):
    def assert_re_setup_preserves_protected_bytes(self, relative: str) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(target)
        self.assertEqual(first_process.returncode, 0, first_process.stderr)
        protected = target / relative
        user_bytes = (
            f"user-maintained canonical bytes for {relative}\n".encode("utf-8")
            + b"\x00\xff"
        )
        protected.write_bytes(user_bytes)

        second_process, result = self.invoke_setup(target)

        self.assertEqual(second_process.returncode, 0, second_process.stderr)
        self.assertNotIn(relative, result["changed_paths"])
        self.assertEqual(protected.read_bytes(), user_bytes)


def make_protected_canonical_test(relative: str):
    def test(self: Phase1ProtectedCanonicalData) -> None:
        self.assert_re_setup_preserves_protected_bytes(relative)

    test.__name__ = (
        "test_re_setup_preserves_"
        + relative.removeprefix(".hive/").replace("/", "_").replace(".", "_")
    )
    return test


for protected_path in PROTECTED_CANONICAL_PATHS:
    setattr(
        Phase1ProtectedCanonicalData,
        make_protected_canonical_test(protected_path).__name__,
        make_protected_canonical_test(protected_path),
    )


class Phase1NoFollowReadBoundary(Phase1CliTestCase):
    def make_fifo_or_skip(self, path: Path) -> None:
        if os.name == "nt" or not hasattr(os, "mkfifo"):
            self.skipTest("named-pipe read evidence is unavailable on this host")
        os.mkfifo(path)

    def assert_fifo_symlink_rejected_without_read(
        self,
        target: Path,
        relative: str,
        *,
        mode: str = "--apply",
        expected_exit: int = 3,
    ) -> None:
        outside_fifo = self.work_root / (
            relative.replace("/", "-").replace(".", "_") + ".fifo"
        )
        self.make_fifo_or_skip(outside_fifo)
        link = target / relative
        link.parent.mkdir(parents=True, exist_ok=True)
        if link.exists() or link.is_symlink():
            link.unlink()
        link.symlink_to(outside_fifo)
        before = snapshot_tree(target)

        try:
            process, result = self.invoke_setup(
                target,
                mode=mode,
                timeout=2.0,
            )
        except subprocess.TimeoutExpired as error:
            self.fail(
                f"setup followed and blocked reading external FIFO via {relative}: {error}"
            )

        self.assertEqual(process.returncode, expected_exit, process.stderr)
        self.assertEqual(result["changed_paths"], [])
        self.assertRegex(
            result["code"],
            r"^hive\.setup-(?:conflict|safety-blocked|verification-failed)$",
        )
        self.assertEqual(snapshot_tree(target), before)

    def test_agents_symlink_is_rejected_before_external_fifo_read(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        self.assert_fifo_symlink_rejected_without_read(target, "AGENTS.md")

    def test_generated_config_symlink_is_rejected_before_external_fifo_read(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(target)
        self.assertEqual(first_process.returncode, 0, first_process.stderr)

        self.assert_fifo_symlink_rejected_without_read(
            target,
            ".hive/config/harness.toml",
            mode="--validate",
        )

    def test_required_role_symlink_is_rejected_before_external_fifo_read(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(target)
        self.assertEqual(first_process.returncode, 0, first_process.stderr)

        self.assert_fifo_symlink_rejected_without_read(
            target,
            ".hive/team/roles/reviewer.md",
            mode="--validate",
            expected_exit=5,
        )


class Phase1DryRunStaging(Phase1CliTestCase):
    def stage_siblings(self, target: Path) -> list[Path]:
        return sorted(target.parent.glob(".aigent-hive-stage-*"))

    def test_dry_run_validates_sibling_staging_without_target_write(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        (target / "user-sentinel.bin").write_bytes(b"user bytes\n")
        before = snapshot_tree(target)

        process, result = self.invoke_setup(target, mode="--dry-run")

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.setup-dry-run-complete")
        self.assertTrue(result["changed_paths"])
        self.assertEqual(snapshot_tree(target), before)
        self.assertEqual(self.stage_siblings(target), [])

    def test_injected_staging_corruption_fails_without_target_write(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        (target / "user-sentinel.bin").write_bytes(b"user bytes\n")
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            mode="--dry-run",
            environment={"HIVE_TEST_STAGING_CORRUPT_AFTER_RENDER": "1"},
        )

        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["code"], "hive.setup-verification-failed")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)
        self.assertEqual(self.stage_siblings(target), [])


class Phase1ActivationRollback(Phase1CliTestCase):
    def changed_answers(self) -> Path:
        path, answers = self.copied_answers("answers-base.yml")
        answers["project_name"] = "phase1-rollback-after-live-replacement"
        write_yaml(path, answers)
        return path

    def invoke_through_sentinel_wrapper(
        self,
        target: Path,
        answers: Path,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object], Path]:
        if os.name == "nt":
            self.skipTest("PID-preserving exec wrapper is POSIX-specific")
        wrapper = self.work_root / "hive-with-user-temp-sentinel.sh"
        pid_file = self.work_root / "wrapper.pid"
        wrapper.write_text(
            "#!/bin/sh\n"
            f"printf '%s\\n' \"$$\" > {str(pid_file)!r}\n"
            f'sentinel="{target}/AGENTS.hive-tmp-$$"\n'
            'mkdir "$sentinel"\n'
            'printf "user sentinel bytes\\n" > "$sentinel/sentinel.txt"\n'
            f"exec {str(self.hive_binary)!r} \"$@\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o700)
        command = [
            str(wrapper),
            "setup",
            "--target",
            str(target),
            "--answers",
            str(answers),
            "--capabilities",
            str(FIXTURE_ROOT / "capabilities-codex-omx.json"),
            "--apply",
            "--output",
            "json",
        ]
        process = subprocess.run(
            command,
            cwd=self.work_root,
            check=False,
            text=True,
            capture_output=True,
        )
        result = json.loads(process.stdout)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"])
        pid = pid_file.read_text(encoding="utf-8").strip()
        return process, result, target / f"AGENTS.hive-tmp-{pid}"

    def test_predictable_pid_temp_user_sentinel_survives_successful_activation(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(target)
        self.assertEqual(first_process.returncode, 0, first_process.stderr)
        process, result, sentinel = self.invoke_through_sentinel_wrapper(
            target,
            self.changed_answers(),
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.setup-complete")
        self.assertTrue(sentinel.is_dir())
        self.assertEqual(
            (sentinel / "sentinel.txt").read_bytes(),
            b"user sentinel bytes\n",
        )
        self.assertIn(
            'project_name = "phase1-rollback-after-live-replacement"',
            (target / ".hive/config/harness.toml").read_text(encoding="utf-8"),
        )

    def test_injected_mid_activation_failure_restores_complete_active_tree(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(target)
        self.assertEqual(first_process.returncode, 0, first_process.stderr)
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            answers=self.changed_answers(),
            environment={"HIVE_TEST_ACTIVATION_FAIL_AFTER": "2"},
        )

        self.assertEqual(process.returncode, 10, process.stderr)
        self.assertEqual(result["code"], "hive.internal-error")
        self.assertEqual(result["changed_paths"], [])
        self.assertIn("rolled back", result["message"])
        self.assertEqual(snapshot_tree(target), before)

    def test_rollback_failure_returns_explicit_non_success_diagnostic(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        first_process, _ = self.invoke_setup(target)
        self.assertEqual(first_process.returncode, 0, first_process.stderr)

        process, result = self.invoke_setup(
            target,
            answers=self.changed_answers(),
            environment={
                "HIVE_TEST_ACTIVATION_FAIL_AFTER": "2",
                "HIVE_TEST_ROLLBACK_FAIL": "1",
            },
        )

        self.assertEqual(process.returncode, 10, process.stderr)
        self.assertEqual(result["status"], "error")
        self.assertEqual(result["code"], "hive.activation-rollback-failed")
        self.assertEqual(result["changed_paths"], [])
        self.assertTrue(
            result["message"].startswith("hive.activation-rollback-failed"),
            result["message"],
        )


class Phase1HookRevocation(Phase1CliTestCase):
    def install_hooks(self) -> Path:
        target = self.work_root / "consumer"
        target.mkdir()
        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        return target

    def assert_tamper_blocks_revocation(self, target: Path) -> None:
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["code"], "hive.setup-conflict")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def test_revocation_dry_run_reports_owned_removals_without_mutation(
        self,
    ) -> None:
        target = self.install_hooks()
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
            capabilities="capabilities-absent.json",
            mode="--dry-run",
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.setup-dry-run-complete")
        self.assertTrue(
            {
                ".hive/config/approved-hooks.yml",
                ".hive/hooks/protect-hive-owned-state",
                ".hive/hooks/checkpoint-reminder",
            }.issubset(set(result["changed_paths"]))
        )
        self.assertEqual(snapshot_tree(target), before)

    def test_reconfigure_revocation_removes_only_hook_ledger_and_descriptors(
        self,
    ) -> None:
        target = self.install_hooks()
        foreign = target / ".hive/hooks/user-sentinel.txt"
        foreign.write_bytes(b"user hook-adjacent bytes\n")

        second_process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(second_process.returncode, 0, second_process.stderr)
        self.assertFalse((target / ".hive/config/approved-hooks.yml").exists())
        self.assertFalse(
            (target / ".hive/hooks/protect-hive-owned-state").exists()
        )
        self.assertFalse((target / ".hive/hooks/checkpoint-reminder").exists())
        self.assertEqual(foreign.read_bytes(), b"user hook-adjacent bytes\n")

        hook_process, hook_result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=FIXTURE_ROOT / "pretool-input.json",
        )
        self.assertEqual(hook_process.returncode, 0, hook_process.stderr)
        self.assertIs(hook_result.get("active"), False)
        self.assertEqual(hook_result.get("decision"), "allow")

    def test_revocation_refuses_tampered_descriptor_without_deleting_bytes(
        self,
    ) -> None:
        target = self.install_hooks()
        descriptor = target / ".hive/hooks/protect-hive-owned-state"
        descriptor.write_bytes(b"user-or-attacker replacement bytes\n")

        self.assert_tamper_blocks_revocation(target)

    def test_revocation_refuses_tampered_ledger_without_deleting_bytes(
        self,
    ) -> None:
        target = self.install_hooks()
        ledger_path = target / ".hive/config/approved-hooks.yml"
        ledger = read_yaml(ledger_path)
        hooks = ledger["hooks"]
        self.assertIsInstance(hooks, list)
        hooks[0]["consent_digest"] = f"sha256:{'0' * 64}"
        write_yaml(ledger_path, ledger)

        self.assert_tamper_blocks_revocation(target)


class Phase1InstalledValidation(Phase1CliTestCase):
    def install(self, *, with_hooks: bool = False) -> Path:
        target = self.work_root / "consumer"
        target.mkdir()
        process, _ = self.invoke_setup(
            target,
            answers=(
                FIXTURE_ROOT / "answers-partial-hooks.yml"
                if with_hooks
                else FIXTURE_ROOT / "answers-base.yml"
            ),
            capabilities=(
                "capabilities-absent.json"
                if with_hooks
                else "capabilities-codex-omx.json"
            ),
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        return target

    def assert_validate_rejects_preserving_tree(
        self,
        target: Path,
        *,
        answers: Path = FIXTURE_ROOT / "answers-base.yml",
        capabilities: str = "capabilities-codex-omx.json",
    ) -> dict[str, object]:
        before = snapshot_tree(target)
        process, result = self.invoke_setup(
            target,
            answers=answers,
            capabilities=capabilities,
            mode="--validate",
        )
        self.assertEqual(process.returncode, 5, process.stderr)
        self.assertEqual(result["status"], "verification-failed")
        self.assertEqual(result["code"], "hive.setup-verification-failed")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)
        return result

    def test_validate_rejects_missing_agents_marker(self) -> None:
        target = self.install()
        (target / "AGENTS.md").write_bytes(b"user text without Hive marker\n")

        self.assert_validate_rejects_preserving_tree(target)

    def test_validate_rejects_missing_required_generated_file(self) -> None:
        target = self.install()
        (target / ".hive/config/knowledge-scope.yml").unlink()

        self.assert_validate_rejects_preserving_tree(target)

    def test_validate_rejects_missing_approved_hook_ledger(self) -> None:
        target = self.install(with_hooks=True)
        (target / ".hive/config/approved-hooks.yml").unlink()

        self.assert_validate_rejects_preserving_tree(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )

    def test_validate_rejects_missing_approved_hook_descriptor(self) -> None:
        target = self.install(with_hooks=True)
        (target / ".hive/hooks/checkpoint-reminder").unlink()

        self.assert_validate_rejects_preserving_tree(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )

    def test_validate_rejects_missing_required_role(self) -> None:
        target = self.install()
        (target / ".hive/team/roles/reviewer.md").unlink()

        self.assert_validate_rejects_preserving_tree(target)

    def test_validate_rejects_harness_version_mismatch(self) -> None:
        target = self.install()
        harness = target / ".hive/config/harness.toml"
        harness.write_text(
            harness.read_text(encoding="utf-8").replace(
                'harness_version = "0.5.0"',
                'harness_version = "0.2.1"',
            ),
            encoding="utf-8",
        )

        self.assert_validate_rejects_preserving_tree(target)

    def test_validate_rejects_source_release_version_mismatch(self) -> None:
        target = self.install()
        harness = target / ".hive/config/harness.toml"
        harness.write_text(
            harness.read_text(encoding="utf-8").replace(
                'source_release_version = "0.5.0"',
                'source_release_version = "0.2.1"',
            ),
            encoding="utf-8",
        )

        self.assert_validate_rejects_preserving_tree(target)

    def test_validate_rejects_corrupted_required_generated_file(self) -> None:
        target = self.install()
        (target / ".hive/config/knowledge-scope.yml").write_bytes(
            b"include: [tampered]\nexclude: []\n"
        )

        self.assert_validate_rejects_preserving_tree(target)

    def test_validate_rejects_supplied_capability_mismatch(self) -> None:
        target = self.install()

        self.assert_validate_rejects_preserving_tree(
            target,
            capabilities="capabilities-unknown.json",
        )

    def test_validate_rejects_malformed_required_role(self) -> None:
        target = self.install()
        (target / ".hive/team/roles/reviewer.md").write_bytes(
            b"---\n{not-json}\n---\nuser body\n"
        )

        self.assert_validate_rejects_preserving_tree(target)

    def test_validate_rejects_revoked_known_hook_descriptor_without_ledger(
        self,
    ) -> None:
        target = self.install()
        hook_directory = target / ".hive/hooks"
        hook_directory.mkdir(parents=True)
        (hook_directory / "protect-hive-owned-state").write_bytes(
            b"stale revoked Hive descriptor bytes\n"
        )

        self.assert_validate_rejects_preserving_tree(target)


class Phase1CliSurface(Phase1CliTestCase):
    def test_help_lists_setup_validate_and_hook_surfaces(self) -> None:
        process = subprocess.run(
            [str(self.hive_binary), "--help"],
            cwd=self.work_root,
            check=False,
            text=True,
            capture_output=True,
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIn("hive setup", process.stdout)
        self.assertIn("--validate", process.stdout)
        self.assertIn("hive hook", process.stdout)

    def test_setup_help_lists_exact_supported_invocation_without_json_failure(
        self,
    ) -> None:
        process = subprocess.run(
            [str(self.hive_binary), "setup", "--help"],
            cwd=self.work_root,
            check=False,
            text=True,
            capture_output=True,
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIn("hive setup", process.stdout)
        self.assertIn("--target <dir>", process.stdout)
        self.assertIn("--answers <yml>", process.stdout)
        self.assertIn("--capabilities <json>", process.stdout)
        self.assertIn("--dry-run|--apply|--validate", process.stdout)
        self.assertIn("--output json", process.stdout)
        with self.assertRaises(json.JSONDecodeError):
            json.loads(process.stdout)

    def test_unknown_action_json_is_schema_valid_and_write_free(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        (target / "sentinel.bin").write_bytes(b"user bytes\n")
        before = snapshot_tree(target)

        process = subprocess.run(
            [str(self.hive_binary), "unknown-action", "--output", "json"],
            cwd=target,
            check=False,
            text=True,
            capture_output=True,
        )

        result = json.loads(process.stdout)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["action"], "UnknownAction")
        self.assertEqual(result["status"], "error")
        self.assertEqual(result["exit_code"], 2)
        self.assertEqual(result["code"], "hive.unknown-action")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(result["evidence"], [])
        self.assertIsNone(result["next_action"])
        self.assertEqual(snapshot_tree(target), before)

    def test_active_protection_hook_blocks_protected_delete(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        setup_process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(setup_process.returncode, 0, setup_process.stderr)
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=FIXTURE_ROOT / "pretool-input.json",
        )

        self.assertNotEqual(process.returncode, 0)
        self.assertIs(result.get("active"), True)
        self.assertEqual(result.get("decision"), "block")
        self.assertEqual(snapshot_tree(target), before)

    def test_inactive_hook_does_not_read_missing_input(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        setup_process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(setup_process.returncode, 0, setup_process.stderr)
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=self.work_root / "must-not-be-read.json",
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)


class Phase1ProtectionHookPaths(Phase1CliTestCase):
    def install_hook(self) -> Path:
        target = self.work_root / "consumer"
        target.mkdir()
        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        return target

    def hook_input(self, name: str, path: str) -> Path:
        input_path = self.work_root / f"{name}.json"
        input_path.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "event": "PreToolUse",
                    "tool": "filesystem-write",
                    "operation": "delete",
                    "path": path,
                },
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )
        return input_path

    def test_target_relative_protected_path_is_blocked(self) -> None:
        target = self.install_hook()
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=self.hook_input(
                "relative-protected",
                ".hive/config/harness.toml",
            ),
        )

        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertIs(result.get("active"), True)
        self.assertEqual(result.get("decision"), "block")
        self.assertEqual(snapshot_tree(target), before)

    def test_inside_target_absolute_protected_path_is_blocked(self) -> None:
        target = self.install_hook()
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=self.hook_input(
                "absolute-protected",
                str((target / ".hive/config/harness.toml").resolve()),
            ),
        )

        self.assertEqual(
            process.returncode,
            3,
            f"stderr={process.stderr!r}\nresult={result!r}\ntarget={target!r}",
        )
        self.assertIs(result.get("active"), True)
        self.assertEqual(result.get("decision"), "block")
        self.assertEqual(snapshot_tree(target), before)

    def test_outside_target_absolute_path_is_non_protected(self) -> None:
        target = self.install_hook()
        outside = self.work_root / "outside-user-file.txt"
        outside.write_bytes(b"outside bytes\n")
        input_path = self.hook_input(
            "absolute-outside",
            str(outside.resolve()),
        )
        before = snapshot_tree(self.work_root)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=input_path,
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIs(result.get("active"), True)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(self.work_root), before)

    def test_parent_traversal_path_is_neutral_inactive_allow(self) -> None:
        target = self.install_hook()
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=self.hook_input(
                "parent-traversal",
                "../outside-user-file.txt",
            ),
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)

    def test_foreign_runtime_namespace_is_neutral_inactive_allow(self) -> None:
        target = self.install_hook()
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=self.hook_input(
                "foreign-runtime",
                ".omx/state/runtime.json",
            ),
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)


if __name__ == "__main__":
    import unittest

    unittest.main()
