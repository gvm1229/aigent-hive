#!/usr/bin/env python3
"""Hostile consent and installed fallback-hook conformance."""

from __future__ import annotations

import hashlib
import json
import os
import shlex
import subprocess
import time
from pathlib import Path

from tests.conformance.phase1_support import (
    EXPECTED_ROOT,
    FIXTURE_ROOT,
    FRESH_HOOK_CAPABILITIES_PATH,
    Phase1CliTestCase,
    read_yaml,
    snapshot_tree,
    write_yaml,
)


class Phase1ConsentHostile(Phase1CliTestCase):
    def assert_skill_field_tamper(
        self,
        field: str,
        replacement: object,
        expected_exit: int,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers("answers-approved-skill.yml")
        approvals = answers["approved_optional_skills"]
        self.assertIsInstance(approvals, list)
        approvals[0][field] = replacement
        write_yaml(answer_path, answers)
        before = snapshot_tree(target)

        process, result = self.invoke_setup(target, answers=answer_path)

        self.assertEqual(process.returncode, expected_exit)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def assert_hook_field_tamper(
        self,
        field: str,
        replacement: object,
        expected_exit: int,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers("answers-partial-hooks.yml")
        approvals = answers["approved_fallback_hooks"]
        self.assertIsInstance(approvals, list)
        approvals[0][field] = replacement
        write_yaml(answer_path, answers)
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            answers=answer_path,
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, expected_exit)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def assert_skill_array_rejected(
        self,
        *,
        requested: list[str],
        approved: list[str],
        literal_digest: str,
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        answer_path, answers = self.copied_answers("answers-approved-skill.yml")
        approvals = answers["approved_optional_skills"]
        self.assertIsInstance(approvals, list)
        approval = approvals[0]
        approval["requested_capabilities"] = requested
        approval["approved_capabilities"] = approved
        approval["consent_digest"] = literal_digest
        write_yaml(answer_path, answers)
        before = snapshot_tree(target)

        process, result = self.invoke_setup(target, answers=answer_path)

        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def test_unsorted_requested_skill_capabilities_are_rejected(self) -> None:
        self.assert_skill_array_rejected(
            requested=["network", "filesystem-read"],
            approved=["filesystem-read"],
            literal_digest=(
                "sha256:b0eb7a7d52d6aed8629f13ef23e919e9"
                "de31dc7153b77bb7be061fa151fcfe90"
            ),
        )

    def test_duplicate_requested_skill_capabilities_are_rejected(self) -> None:
        self.assert_skill_array_rejected(
            requested=["filesystem-read", "filesystem-read", "network"],
            approved=["filesystem-read"],
            literal_digest=(
                "sha256:74556205f96ea8f45b1d422eab207a35"
                "cb260e262192873269ed4df8f03a93cc"
            ),
        )

    def test_unsorted_approved_skill_capabilities_are_rejected(self) -> None:
        self.assert_skill_array_rejected(
            requested=["filesystem-read", "network", "shell"],
            approved=["shell", "filesystem-read"],
            literal_digest=(
                "sha256:820d35ba189f3e5fca0890011d2a34c6"
                "e36d1632349405615ff0ca8c3e4193a7"
            ),
        )

    def test_duplicate_approved_skill_capabilities_are_rejected(self) -> None:
        self.assert_skill_array_rejected(
            requested=["filesystem-read", "network"],
            approved=["filesystem-read", "filesystem-read"],
            literal_digest=(
                "sha256:5dc10c48146613b8a01bb4edd7a2c08f"
                "ca188d27b993d99faba863a80c2b5fcb"
            ),
        )

    def test_skill_capability_grant_outside_requested_subset_is_rejected(
        self,
    ) -> None:
        self.assert_skill_array_rejected(
            requested=["filesystem-read", "network"],
            approved=["filesystem-read", "shell"],
            literal_digest=(
                "sha256:c211b2b97ab2c201f5dbf486bd80eeb8"
                "fabedb3f0a9bcd1f4d5f93f4c79d0260"
            ),
        )

    def test_skill_timestamp_without_seconds_is_rejected(self) -> None:
        self.assert_skill_field_tamper(
            "approved_at",
            "2026-07-23T00:00Z",
            2,
        )

    def test_skill_timestamp_with_fractional_seconds_is_rejected(self) -> None:
        self.assert_skill_field_tamper(
            "approved_at",
            "2026-07-23T00:00:00.000Z",
            2,
        )

    def test_skill_timestamp_with_offset_is_rejected(self) -> None:
        self.assert_skill_field_tamper(
            "approved_at",
            "2026-07-23T09:00:00+09:00",
            2,
        )

    def test_hook_timestamp_without_seconds_is_rejected(self) -> None:
        self.assert_hook_field_tamper(
            "approved_at",
            "2026-07-23T00:00Z",
            2,
        )

    def test_hook_timestamp_with_fractional_seconds_is_rejected(self) -> None:
        self.assert_hook_field_tamper(
            "approved_at",
            "2026-07-23T00:00:00.000Z",
            2,
        )

    def test_hook_timestamp_with_offset_is_rejected(self) -> None:
        self.assert_hook_field_tamper(
            "approved_at",
            "2026-07-23T09:00:00+09:00",
            2,
        )


SKILL_FIELD_TAMPERS = {
    "consent_version": (2, 2),
    "name": ("changed-name", 3),
    "source": ("https://example.invalid/changed-source", 3),
    "revision": ("v2.0.0", 3),
    "content_digest": (f"sha256:{'4' * 64}", 3),
    "requested_capabilities": (["filesystem-read", "network", "shell"], 3),
    "approved_capabilities": ([], 3),
    "approved_at": ("2026-07-23T00:00:01Z", 3),
    "consent_digest": (f"sha256:{'5' * 64}", 3),
}


def make_skill_tamper_test(
    field: str,
    replacement: object,
    expected_exit: int,
):
    def test(self: Phase1ConsentHostile) -> None:
        self.assert_skill_field_tamper(field, replacement, expected_exit)

    test.__name__ = f"test_skill_{field}_tamper_invalidates_consent"
    return test


for skill_field, (skill_replacement, skill_exit) in SKILL_FIELD_TAMPERS.items():
    setattr(
        Phase1ConsentHostile,
        f"test_skill_{skill_field}_tamper_invalidates_consent",
        make_skill_tamper_test(skill_field, skill_replacement, skill_exit),
    )


HOOK_FIELD_TAMPERS = {
    "consent_version": (2, 2),
    "capability": ("update-integrity-guard", 2),
    "event": ("PostToolUse", 2),
    "path": (".hive/hooks/renamed-protect-hook", 2),
    "command": (
        "hive hook --capability protect-hive-owned-state "
        "--event PreToolUse --output human",
        2,
    ),
    "content_digest": (f"sha256:{'6' * 64}", 3),
    "approved_at": ("2026-07-23T00:00:01Z", 3),
    "consent_digest": (f"sha256:{'7' * 64}", 3),
}


def make_hook_tamper_test(
    field: str,
    replacement: object,
    expected_exit: int,
):
    def test(self: Phase1ConsentHostile) -> None:
        self.assert_hook_field_tamper(field, replacement, expected_exit)

    test.__name__ = f"test_hook_{field}_tamper_invalidates_consent"
    return test


for hook_field, (hook_replacement, hook_exit) in HOOK_FIELD_TAMPERS.items():
    setattr(
        Phase1ConsentHostile,
        f"test_hook_{hook_field}_tamper_invalidates_consent",
        make_hook_tamper_test(hook_field, hook_replacement, hook_exit),
    )


class Phase1InstalledHookConformance(Phase1CliTestCase):
    def invoke_stored_hook_command(
        self,
        target: Path,
        descriptor_path: str,
        *,
        input_text: str,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        descriptor = json.loads((target / descriptor_path).read_text(encoding="utf-8"))
        arguments = shlex.split(descriptor["command"])
        self.assertEqual(arguments[:2], ["hive", "hook"])
        self.assertIn(
            FRESH_HOOK_CAPABILITIES_PATH.as_posix(),
            arguments,
        )
        process = subprocess.run(
            [str(self.hive_binary), *arguments[1:]],
            cwd=target,
            check=False,
            text=True,
            input=input_text,
            capture_output=True,
            env={**os.environ, "PATH": ""},
        )
        result = json.loads(process.stdout)
        self.assertIsInstance(result, dict)
        return process, result

    def install_approved_hooks(self) -> Path:
        target = self.work_root / "consumer"
        target.mkdir()
        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(process.returncode, 0)
        return target

    def install_without_hooks(self) -> Path:
        target = self.work_root / "consumer"
        target.mkdir()
        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(process.returncode, 0)
        return target

    def test_installed_hook_descriptors_match_exact_approved_bytes(self) -> None:
        target = self.install_approved_hooks()

        expected = {
            ".hive/hooks/protect-hive-owned-state": (
                EXPECTED_ROOT / "hook-protect-hive-owned-state.json"
            ).read_bytes(),
            ".hive/hooks/checkpoint-reminder": (
                EXPECTED_ROOT / "hook-checkpoint-reminder-stop.json"
            ).read_bytes(),
        }

        self.assertEqual(
            {
                path: (target / path).read_bytes()
                for path in sorted(expected)
            },
            expected,
        )

    def test_stored_command_uses_ephemeral_fresh_evidence_or_stays_inert(
        self,
    ) -> None:
        target = self.install_approved_hooks()
        runtime_path = target / FRESH_HOOK_CAPABILITIES_PATH
        self.assertFalse(runtime_path.exists())
        runtime_path.parent.mkdir(parents=True)
        fresh_absent = (FIXTURE_ROOT / "capabilities-absent.json").read_bytes()
        pretool_input = (FIXTURE_ROOT / "pretool-input.json").read_text(
            encoding="utf-8"
        )

        runtime_path.write_bytes(fresh_absent)
        active_process, active_result = self.invoke_stored_hook_command(
            target,
            ".hive/hooks/protect-hive-owned-state",
            input_text=pretool_input,
        )
        self.assertEqual(active_process.returncode, 3, active_process.stderr)
        self.assertIs(active_result.get("active"), True)
        self.assertEqual(active_result.get("decision"), "block")

        runtime_path.unlink()
        missing_process, missing_result = self.invoke_stored_hook_command(
            target,
            ".hive/hooks/protect-hive-owned-state",
            input_text="{must-not-be-read\n",
        )
        self.assertEqual(missing_process.returncode, 0, missing_process.stderr)
        self.assertIs(missing_result.get("active"), False)
        self.assertEqual(missing_result.get("decision"), "allow")
        self.assertNotIn("invalid hook input", missing_process.stderr)

        runtime_path.write_bytes(fresh_absent)
        stale_time = time.time() - 61
        os.utime(runtime_path, (stale_time, stale_time))
        stale_process, stale_result = self.invoke_stored_hook_command(
            target,
            ".hive/hooks/protect-hive-owned-state",
            input_text="{must-not-be-read\n",
        )
        self.assertEqual(stale_process.returncode, 0, stale_process.stderr)
        self.assertIs(stale_result.get("active"), False)
        self.assertEqual(stale_result.get("decision"), "allow")
        self.assertNotIn("invalid hook input", stale_process.stderr)

        runtime_path.write_text("{malformed\n", encoding="utf-8")
        malformed_process, malformed_result = self.invoke_stored_hook_command(
            target,
            ".hive/hooks/protect-hive-owned-state",
            input_text="{must-not-be-read\n",
        )
        self.assertEqual(
            malformed_process.returncode,
            0,
            malformed_process.stderr,
        )
        self.assertIs(malformed_result.get("active"), False)
        self.assertEqual(malformed_result.get("decision"), "allow")
        self.assertNotIn("invalid hook input", malformed_process.stderr)

        runtime_path.unlink()
        stop_process, stop_result = self.invoke_stored_hook_command(
            target,
            ".hive/hooks/checkpoint-reminder",
            input_text="{must-not-be-read\n",
        )
        self.assert_neutral_stop(stop_process, stop_result)
        self.assertEqual(stop_process.stderr, "")

    def test_fresh_absent_matrix_must_match_the_ledger_bound_resolution(
        self,
    ) -> None:
        target = self.install_approved_hooks()
        different_absent = json.loads(
            (FIXTURE_ROOT / "capabilities-absent.json").read_text(
                encoding="utf-8"
            )
        )
        different_absent["host_version"] = "different-valid-absent-evidence"
        digest_payload = {
            key: value
            for key, value in different_absent.items()
            if key != "evidence_digest"
        }
        canonical = json.dumps(
            digest_payload,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ).encode()
        different_absent["evidence_digest"] = (
            f"sha256:{hashlib.sha256(canonical).hexdigest()}"
        )
        different_path = self.work_root / "different-absent.json"
        different_path.write_text(
            json.dumps(different_absent, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        malformed_input = self.work_root / "must-not-be-read.json"
        malformed_input.write_text("{must-not-be-read\n", encoding="utf-8")

        inert_process, inert_result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            capabilities=different_path,
            input_path=malformed_input,
        )

        self.assertEqual(inert_process.returncode, 0, inert_process.stderr)
        self.assertIs(inert_result.get("active"), False)
        self.assertEqual(inert_result.get("decision"), "allow")
        self.assertNotIn("invalid hook input", inert_process.stderr)

        active_process, active_result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=FIXTURE_ROOT / "pretool-input.json",
        )
        self.assertEqual(active_process.returncode, 3, active_process.stderr)
        self.assertIs(active_result.get("active"), True)
        self.assertEqual(active_result.get("decision"), "block")

    def test_approved_protection_hook_blocks_protected_delete(self) -> None:
        target = self.install_approved_hooks()
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=FIXTURE_ROOT / "pretool-input.json",
        )

        self.assertEqual(process.returncode, 3)
        self.assertIs(result.get("active"), True)
        self.assertEqual(result.get("decision"), "block")
        self.assertEqual(snapshot_tree(target), before)

    def test_unapproved_non_stop_hook_activation_is_zero(self) -> None:
        target = self.install_without_hooks()
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=FIXTURE_ROOT / "pretool-input.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)

    def test_tampered_hook_ledger_non_stop_activation_is_zero(self) -> None:
        target = self.install_approved_hooks()
        ledger_path = target / ".hive/config/approved-hooks.yml"
        ledger = read_yaml(ledger_path)
        hooks = ledger["hooks"]
        self.assertIsInstance(hooks, list)
        hooks[0]["consent_digest"] = f"sha256:{'8' * 64}"
        write_yaml(ledger_path, ledger)
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=FIXTURE_ROOT / "pretool-input.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)

    def test_tampered_hook_descriptor_non_stop_activation_is_zero(self) -> None:
        target = self.install_approved_hooks()
        descriptor = target / ".hive/hooks/protect-hive-owned-state"
        descriptor.write_bytes(descriptor.read_bytes() + b"tampered\n")
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=FIXTURE_ROOT / "pretool-input.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)

    def test_malformed_pretool_json_is_neutral_inactive_allow(self) -> None:
        target = self.install_approved_hooks()
        malformed = self.work_root / "malformed-pretool.json"
        malformed.write_text("{not-json\n", encoding="utf-8")
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=malformed,
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)

    def test_missing_pretool_field_is_neutral_inactive_allow(self) -> None:
        target = self.install_approved_hooks()
        incomplete = self.work_root / "incomplete-pretool.json"
        incomplete.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "event": "PreToolUse",
                    "tool": "filesystem-write",
                    "operation": "delete",
                }
            )
            + "\n",
            encoding="utf-8",
        )
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            input_path=incomplete,
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)

    def test_malformed_posttool_json_is_neutral_inactive_allow(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        setup_process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-all-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(setup_process.returncode, 0, setup_process.stderr)
        malformed = self.work_root / "malformed-posttool.json"
        malformed.write_text("[not-an-object]\n", encoding="utf-8")
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="derived-state-invalidation",
            event="PostToolUse",
            input_path=malformed,
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(snapshot_tree(target), before)

    def test_fresh_non_absent_matrix_is_inert_before_input_or_installed_state(
        self,
    ) -> None:
        target = self.install_approved_hooks()
        (target / ".hive/config/approved-hooks.yml").write_text(
            "hooks: [must-not-be-read\n",
            encoding="utf-8",
        )
        before = snapshot_tree(target)

        for capabilities in (
            "capabilities-codex-omx.json",
            "capabilities-incompatible.json",
            "capabilities-unknown.json",
        ):
            with self.subTest(capabilities=capabilities):
                process, result = self.invoke_hook(
                    target,
                    capability="protect-hive-owned-state",
                    event="PreToolUse",
                    capabilities=capabilities,
                    input_path=self.work_root / "must-not-be-read.json",
                )

                self.assertEqual(process.returncode, 0, process.stderr)
                self.assertIs(result.get("active"), False)
                self.assertEqual(result.get("decision"), "allow")
                self.assertEqual(process.stderr, "")
                self.assertEqual(snapshot_tree(target), before)

    def test_missing_fresh_matrix_is_inert_before_input_or_installed_state(
        self,
    ) -> None:
        target = self.install_approved_hooks()
        (target / ".hive/config/approved-hooks.yml").write_text(
            "hooks: [must-not-be-read\n",
            encoding="utf-8",
        )
        before = snapshot_tree(target)

        process, result = self.invoke_hook(
            target,
            capability="protect-hive-owned-state",
            event="PreToolUse",
            capabilities=None,
            input_path=self.work_root / "must-not-be-read.json",
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIs(result.get("active"), False)
        self.assertEqual(result.get("decision"), "allow")
        self.assertEqual(process.stderr, "")
        self.assertEqual(snapshot_tree(target), before)

    def test_approved_stop_is_recursively_neutral(self) -> None:
        target = self.install_approved_hooks()

        process, result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
        )

        self.assert_neutral_stop(process, result)

    def test_unapproved_stop_is_recursively_neutral(self) -> None:
        target = self.install_without_hooks()

        process, result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
        )

        self.assert_neutral_stop(process, result)

    def test_tampered_stop_descriptor_is_recursively_neutral(self) -> None:
        target = self.install_approved_hooks()
        descriptor = target / ".hive/hooks/checkpoint-reminder"
        descriptor.write_bytes(b"tampered\n")

        process, result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
        )

        self.assert_neutral_stop(process, result)

    def test_malformed_stop_ledger_is_recursively_neutral(self) -> None:
        target = self.install_approved_hooks()
        (target / ".hive/config/approved-hooks.yml").write_text(
            "hooks: [unterminated\n",
            encoding="utf-8",
        )

        process, result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
        )

        self.assert_neutral_stop(process, result)

    def test_malformed_stop_input_is_recursively_neutral(self) -> None:
        target = self.install_approved_hooks()
        malformed = self.work_root / "malformed-input.json"
        malformed.write_text("{not-json\n", encoding="utf-8")

        process, result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
            input_path=malformed,
        )

        self.assert_neutral_stop(process, result)

    def test_stop_replay_is_neutral_and_does_not_duplicate_mutation(self) -> None:
        target = self.install_approved_hooks()
        before = snapshot_tree(target)

        first_process, first_result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
        )
        second_process, second_result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
        )

        self.assert_neutral_stop(first_process, first_result)
        self.assert_neutral_stop(second_process, second_result)
        self.assertEqual(snapshot_tree(target), before)

    def test_stop_input_error_is_recursively_neutral(self) -> None:
        target = self.install_approved_hooks()

        process, result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
            input_path=self.work_root / "missing-input.json",
        )

        self.assert_neutral_stop(process, result)

    def test_non_absent_resolution_stop_is_recursively_neutral(self) -> None:
        target = self.install_approved_hooks()
        available = json.loads(
            (FIXTURE_ROOT / "capabilities-codex-omx.json").read_text(
                encoding="utf-8"
            )
        )
        write_yaml(target / ".hive/config/capability-resolution.yml", available)

        process, result = self.invoke_hook(
            target,
            capability="checkpoint-reminder",
            event="Stop",
        )

        self.assert_neutral_stop(process, result)
