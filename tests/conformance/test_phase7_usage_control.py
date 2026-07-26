#!/usr/bin/env python3
"""Hostile black-box conformance for the shipping usage-control surface."""

from __future__ import annotations

import hashlib
import json
import os
import stat
import subprocess
import time
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.phase1_support import (
    ACTION_RESULT_SCHEMA,
    Phase1CliTestCase,
    REPOSITORY_ROOT,
    snapshot_tree,
)

RAW_ACCOUNT = "usage-guard@example.invalid"
ACCOUNT_DIGEST = "sha256:" + hashlib.sha256(RAW_ACCOUNT.encode()).hexdigest()
CODEXBAR_FIXTURE = (
    REPOSITORY_ROOT / "tests/fixtures/phase5/usage/codexbar_fixture.py"
).read_text(encoding="utf-8")
CODEX_UNSUPPORTED_FIXTURE = (
    REPOSITORY_ROOT / "tests/fixtures/phase5/usage/codex_unsupported_fixture.py"
).read_text(encoding="utf-8")

HARNESS_CONFIG = """\
schema_version = 1
harness_version = "0.7.0"
source_release_version = "0.7.0"
project_name = "usage-control"
primary_host = "codex"
usage_stop_remaining_percent = 10
foreign_preserved = "exact bytes"
"""


class ShippingUsageControlConformance(Phase1CliTestCase):
    def setUp(self) -> None:
        super().setUp()
        self.consumer = self.work_root / "consumer"
        config = self.consumer / ".hive/config"
        config.mkdir(parents=True)
        (config / "harness.toml").write_text(HARNESS_CONFIG, encoding="utf-8")
        (self.consumer / "sentinel.bin").write_bytes(b"user-owned\n")
        self.fake_bin = self.work_root / "fake-bin"
        self.fake_bin.mkdir()
        self._write_fake_codex_unsupported()
        self._write_fake_codexbar()

    def _write_fake_codex_unsupported(self) -> None:
        if os.name == "nt":
            script = self.fake_bin / "codex.cmd"
            python = subprocess.list2cmdline([os.sys.executable])
            script.write_text(
                f'@{python} "%~dp0\\codex.py" %*\r\n',
                encoding="utf-8",
            )
            (self.fake_bin / "codex.py").write_text(
                CODEX_UNSUPPORTED_FIXTURE,
                encoding="utf-8",
            )
            return
        script = self.fake_bin / "codex"
        script.write_text(
            f"#!{os.sys.executable}\n{CODEX_UNSUPPORTED_FIXTURE}",
            encoding="utf-8",
        )
        script.chmod(script.stat().st_mode | stat.S_IXUSR)

    def _write_fake_codexbar(self) -> None:
        if os.name == "nt":
            script = self.fake_bin / "codexbar.cmd"
            python = subprocess.list2cmdline([os.sys.executable])
            script.write_text(
                f'@{python} "%~dp0\\codexbar.py" %*\r\n',
                encoding="utf-8",
            )
            (self.fake_bin / "codexbar.py").write_text(
                CODEXBAR_FIXTURE,
                encoding="utf-8",
            )
            return
        script = self.fake_bin / "codexbar"
        script.write_text(
            f"#!{os.sys.executable}\n{CODEXBAR_FIXTURE}",
            encoding="utf-8",
        )
        script.chmod(script.stat().st_mode | stat.S_IXUSR)

    def invoke(
        self,
        *arguments: str,
        sensor_case: str | None = None,
        extra_environment: dict[str, str] | None = None,
        stdin: str | None = None,
    ) -> tuple[object, dict[str, object]]:
        environment = {
            **os.environ,
            "PATH": str(self.fake_bin) + os.pathsep + os.environ.get("PATH", ""),
            **(extra_environment or {}),
        }
        if sensor_case is not None:
            environment["FAKE_CODEXBAR_CASE"] = sensor_case
        process = subprocess.run(
            [str(self.hive_binary), *arguments, "--output", "json"],
            cwd=self.consumer,
            check=False,
            text=True,
            capture_output=True,
            input=stdin,
            timeout=5,
            env=environment,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"stdout must contain exactly one JSON result: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        return process, result

    def write_halt(
        self,
        *,
        session_id: str,
        process_id: int,
        decision: str = "halted",
    ) -> Path:
        digest = hashlib.sha256(b"codex\0" + session_id.encode()).hexdigest()
        path = (
            self.consumer
            / ".hive/runtime/usage-guard/sessions"
            / digest
            / "halt.json"
        )
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "host_scope": "codex",
                    "session_id_digest": f"sha256:{digest}",
                    "process_id": process_id,
                    "decision": decision,
                    "selected_window": "session",
                    "threshold_remaining_percent": 10,
                    "measured_at": 1_750_000_000,
                    "evidence_digest": "sha256:" + "a" * 64,
                    "revision": 1,
                },
                separators=(",", ":"),
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        return path

    def assert_result(
        self,
        process: object,
        result: dict[str, object],
        *,
        action: str,
        exit_code: int,
        status: str,
        code: str,
    ) -> None:
        self.assertEqual(process.returncode, exit_code, process.stderr)
        self.assertEqual(result.get("action"), action)
        self.assertEqual(result.get("exit_code"), exit_code)
        self.assertEqual(result.get("status"), status)
        self.assertEqual(result.get("code"), code)

    def assert_preflight_only(self, result: dict[str, object]) -> None:
        data = result.get("data")
        self.assertIsInstance(data, dict)
        assert isinstance(data, dict)
        self.assertEqual(data.get("scope"), "automatic-dispatch-preflight")
        self.assertIs(data.get("authorizes_dispatch"), False)

    def test_status_defaults_to_enabled_without_creating_runtime_state(self) -> None:
        before = snapshot_tree(self.consumer)

        process, result = self.invoke(
            "usage",
            "status",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "101",
        )

        self.assert_result(
            process,
            result,
            action="ShowUsageStatus",
            exit_code=0,
            status="success",
            code="hive.usage-status",
        )
        self.assertEqual(result["data"]["threshold_remaining_percent"], 10)
        self.assertTrue(result["data"]["guard_enabled"])
        self.assertEqual(result["data"]["session_override"], "absent")
        self.assertEqual(result["data"]["halt_marker"], "absent")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.consumer), before)

    def test_threshold_updates_only_the_owned_root_key_and_is_idempotent(self) -> None:
        process, result = self.invoke(
            "usage",
            "threshold",
            "--target",
            str(self.consumer),
            "--remaining-percent",
            "17",
        )

        self.assert_result(
            process,
            result,
            action="SetUsageThreshold",
            exit_code=0,
            status="success",
            code="hive.usage-threshold-updated",
        )
        config = (self.consumer / ".hive/config/harness.toml").read_text(
            encoding="utf-8"
        )
        self.assertEqual(
            config,
            HARNESS_CONFIG.replace(
                "usage_stop_remaining_percent = 10",
                "usage_stop_remaining_percent = 17",
            ),
        )
        self.assertEqual(
            result["changed_paths"], [".hive/config/harness.toml"]
        )
        self.assertEqual((self.consumer / "sentinel.bin").read_bytes(), b"user-owned\n")

        repeated, repeated_result = self.invoke(
            "usage",
            "threshold",
            "--target",
            str(self.consumer),
            "--remaining-percent",
            "17",
        )
        self.assert_result(
            repeated,
            repeated_result,
            action="SetUsageThreshold",
            exit_code=0,
            status="success",
            code="hive.usage-threshold-unchanged",
        )
        self.assertEqual(repeated_result["changed_paths"], [])

    def test_threshold_rejects_invalid_primary_host_without_mutation(self) -> None:
        config = self.consumer / ".hive/config/harness.toml"
        invalid = HARNESS_CONFIG.replace(
            'primary_host = "codex"',
            'primary_host = "unsupported"',
        )
        config.write_text(invalid, encoding="utf-8")

        process, result = self.invoke(
            "usage",
            "threshold",
            "--target",
            str(self.consumer),
            "--remaining-percent",
            "17",
        )

        self.assert_result(
            process,
            result,
            action="SetUsageThreshold",
            exit_code=3,
            status="blocked",
            code="hive.usage-control-blocked",
        )
        self.assertEqual(config.read_text(encoding="utf-8"), invalid)

    def test_threshold_rejects_oversized_config_without_mutation(self) -> None:
        config = self.consumer / ".hive/config/harness.toml"
        oversized = HARNESS_CONFIG.encode() + b"#" + b"x" * (64 * 1024)
        config.write_bytes(oversized)

        process, result = self.invoke(
            "usage",
            "threshold",
            "--target",
            str(self.consumer),
            "--remaining-percent",
            "17",
        )

        self.assert_result(
            process,
            result,
            action="SetUsageThreshold",
            exit_code=2,
            status="error",
            code="hive.invalid-input",
        )
        self.assertEqual(config.read_bytes(), oversized)

    def test_session_disable_requires_confirmation_and_never_persists_raw_id(self) -> None:
        rejected, rejected_result = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "private-session-id",
            "--process-id",
            "202",
            "--action",
            "disable",
        )
        self.assert_result(
            rejected,
            rejected_result,
            action="ControlUsageSession",
            exit_code=2,
            status="error",
            code="hive.invalid-input",
        )

        process, result = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "private-session-id",
            "--process-id",
            "202",
            "--action",
            "disable",
            "--confirm-session-disable",
        )
        self.assert_result(
            process,
            result,
            action="ControlUsageSession",
            exit_code=0,
            status="success",
            code="hive.usage-session-disabled",
        )
        runtime = self.consumer / ".hive/runtime/usage-guard"
        persisted = b"".join(
            path.read_bytes() for path in runtime.rglob("*") if path.is_file()
        )
        self.assertNotIn(b"private-session-id", persisted)

        status, status_result = self.invoke(
            "usage",
            "status",
            "--target",
            str(self.consumer),
            "--session-id",
            "private-session-id",
            "--process-id",
            "202",
        )
        self.assert_result(
            status,
            status_result,
            action="ShowUsageStatus",
            exit_code=0,
            status="success",
            code="hive.usage-status",
        )
        self.assertFalse(status_result["data"]["guard_enabled"])
        self.assertEqual(status_result["data"]["session_override"], "current")

    def test_override_is_not_transferred_to_another_session_or_process(self) -> None:
        disabled, _ = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "303",
            "--action",
            "disable",
            "--confirm-session-disable",
        )
        self.assertEqual(disabled.returncode, 0, disabled.stderr)

        for session_id, process_id, expected_override in (
            ("session-b", "303", "absent"),
            ("session-a", "404", "stale"),
        ):
            with self.subTest(session_id=session_id, process_id=process_id):
                process, result = self.invoke(
                    "usage",
                    "status",
                    "--target",
                    str(self.consumer),
                    "--session-id",
                    session_id,
                    "--process-id",
                    process_id,
                )
                self.assertEqual(process.returncode, 0, process.stderr)
                self.assertTrue(result["data"]["guard_enabled"])
                self.assertEqual(
                    result["data"]["session_override"], expected_override
                )

    def test_current_halt_blocks_every_status_gate_until_explicit_bypass(self) -> None:
        self.write_halt(session_id="session-a", process_id=606)

        for _ in range(2):
            process, result = self.invoke(
                "usage",
                "status",
                "--target",
                str(self.consumer),
                "--session-id",
                "session-a",
                "--process-id",
                "606",
            )
            self.assert_result(
                process,
                result,
                action="ShowUsageStatus",
                exit_code=3,
                status="blocked",
                code="hive.usage-session-halted",
            )
            self.assertTrue(result["data"]["guard_enabled"])
            self.assertEqual(result["data"]["halt_marker"], "current")

        disabled, _ = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "606",
            "--action",
            "disable",
            "--confirm-session-disable",
        )
        self.assertEqual(disabled.returncode, 0, disabled.stderr)

        bypassed, bypassed_result = self.invoke(
            "usage",
            "status",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "606",
        )
        self.assert_result(
            bypassed,
            bypassed_result,
            action="ShowUsageStatus",
            exit_code=0,
            status="success",
            code="hive.usage-status",
        )
        self.assertFalse(bypassed_result["data"]["guard_enabled"])
        self.assertEqual(bypassed_result["data"]["halt_marker"], "current")

        enabled, _ = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "606",
            "--action",
            "enable",
        )
        self.assertEqual(enabled.returncode, 0, enabled.stderr)
        blocked_again, blocked_result = self.invoke(
            "usage",
            "status",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "606",
        )
        self.assertEqual(blocked_again.returncode, 3, blocked_again.stderr)
        self.assertEqual(blocked_result["code"], "hive.usage-session-halted")

    def test_halt_marker_is_not_replayed_to_another_binding(self) -> None:
        self.write_halt(session_id="session-a", process_id=707)
        for session_id, process_id, expected in (
            ("session-b", "707", "absent"),
            ("session-a", "808", "stale"),
        ):
            process, result = self.invoke(
                "usage",
                "status",
                "--target",
                str(self.consumer),
                "--session-id",
                session_id,
                "--process-id",
                process_id,
            )
            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertTrue(result["data"]["guard_enabled"])
            self.assertEqual(result["data"]["halt_marker"], expected)

    def test_enforce_refuses_a_process_replayed_halt_marker_without_sensor_use(
        self,
    ) -> None:
        self.write_halt(session_id="replayed-session", process_id=707)
        sensor_log = self.work_root / "replay-sensor.log"
        process, result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "replayed-session",
            "--process-id",
            "808",
            sensor_case="allow",
            extra_environment={"FAKE_CODEXBAR_LOG": str(sensor_log)},
        )

        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["code"], "hive.usage-control-blocked")
        self.assertFalse(sensor_log.exists())

    def test_enable_and_toggle_apply_only_to_the_current_binding(self) -> None:
        disabled, _ = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "505",
            "--action",
            "disable",
            "--confirm-session-disable",
        )
        self.assertEqual(disabled.returncode, 0, disabled.stderr)

        enabled, enabled_result = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "505",
            "--action",
            "enable",
        )
        self.assert_result(
            enabled,
            enabled_result,
            action="ControlUsageSession",
            exit_code=0,
            status="success",
            code="hive.usage-session-enabled",
        )

        missing_confirmation, result = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "505",
            "--action",
            "toggle",
        )
        self.assert_result(
            missing_confirmation,
            result,
            action="ControlUsageSession",
            exit_code=2,
            status="error",
            code="hive.invalid-input",
        )

        toggled_off, toggled_off_result = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "505",
            "--action",
            "toggle",
            "--confirm-session-disable",
        )
        self.assert_result(
            toggled_off,
            toggled_off_result,
            action="ControlUsageSession",
            exit_code=0,
            status="success",
            code="hive.usage-session-disabled",
        )

        toggled_on, toggled_on_result = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-a",
            "--process-id",
            "505",
            "--action",
            "toggle",
        )
        self.assert_result(
            toggled_on,
            toggled_on_result,
            action="ControlUsageSession",
            exit_code=0,
            status="success",
            code="hive.usage-session-enabled",
        )

    def test_enforce_allows_a_fresh_session_window_without_persisting_sensor_payload(
        self,
    ) -> None:
        process, result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-allow",
            "--process-id",
            "901",
            "--account-digest",
            ACCOUNT_DIGEST,
            sensor_case="allow",
        )

        self.assert_result(
            process,
            result,
            action="CheckUsage",
            exit_code=0,
            status="success",
            code="hive.usage-allowed",
        )
        self.assertEqual(result["data"]["selected_window"], "session")
        self.assertEqual(result["data"]["host_scope"], "codex")
        self.assert_preflight_only(result)
        self.assertFalse(
            (self.consumer / ".hive/runtime/usage-guard").exists()
        )
        self.assertNotIn(RAW_ACCOUNT, process.stdout)

    def test_claude_capture_is_sanitized_session_bound_and_native_limited(self) -> None:
        config = self.consumer / ".hive/config/harness.toml"
        config.write_text(
            config.read_text(encoding="utf-8").replace(
                'primary_host = "codex"',
                'primary_host = "claude"',
            ),
            encoding="utf-8",
        )
        raw_session = "claude-private-session"
        reset = int(time.time()) + 3600
        capture, capture_result = self.invoke(
            "usage",
            "capture",
            "--host",
            "claude",
            "--target",
            str(self.consumer),
            "--stdin-json",
            stdin=json.dumps(
                {
                    "session_id": raw_session,
                    "version": "2.1.90",
                    "cwd": "/private/repository",
                    "transcript_path": "/private/transcript.jsonl",
                    "rate_limits": {
                        "five_hour": {
                            "used_percentage": 90,
                            "resets_at": reset,
                        },
                        "seven_day": {
                            "used_percentage": 1,
                            "resets_at": reset + 3600,
                        },
                    },
                }
            ),
        )
        self.assert_result(
            capture,
            capture_result,
            action="CaptureUsage",
            exit_code=0,
            status="success",
            code="hive.usage-capture-recorded",
        )
        capture_path = self.consumer / capture_result["changed_paths"][0]
        persisted = capture_path.read_text(encoding="utf-8")
        self.assertNotIn(raw_session, persisted)
        self.assertNotIn("/private/", persisted)

        sensor_log = self.work_root / "claude-sensor.log"
        enforced, enforced_result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            raw_session,
            "--process-id",
            "990",
            sensor_case="allow",
            extra_environment={
                "FAKE_CODEXBAR_LOG": str(sensor_log),
                "FAKE_CODEXBAR_PROVIDER": "claude",
            },
        )
        self.assert_result(
            enforced,
            enforced_result,
            action="CheckUsage",
            exit_code=3,
            status="blocked",
            code="hive.usage-limited",
        )
        self.assertEqual(enforced_result["data"]["selected_window"], "session")
        self.assertFalse(sensor_log.exists(), "native limited must not call CodexBar")

    def test_antigravity_native_unsupported_uses_codexbar_fallback(self) -> None:
        config = self.consumer / ".hive/config/harness.toml"
        config.write_text(
            config.read_text(encoding="utf-8").replace(
                'primary_host = "codex"',
                'primary_host = "antigravity"',
            ),
            encoding="utf-8",
        )
        sensor_log = self.work_root / "antigravity-sensor.log"

        process, result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "antigravity-session",
            "--process-id",
            "991",
            "--account-digest",
            ACCOUNT_DIGEST,
            sensor_case="allow",
            extra_environment={
                "FAKE_CODEXBAR_LOG": str(sensor_log),
                "FAKE_CODEXBAR_PROVIDER": "antigravity",
            },
        )

        self.assert_result(
            process,
            result,
            action="CheckUsage",
            exit_code=0,
            status="success",
            code="hive.usage-allowed",
        )
        self.assertEqual(result["data"]["host_scope"], "antigravity")
        self.assertEqual(len(sensor_log.read_text(encoding="utf-8").splitlines()), 2)

    def test_enforce_creates_a_latched_marker_and_repeat_skips_the_sensor(
        self,
    ) -> None:
        sensor_log = self.work_root / "sensor.log"
        arguments = (
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-limited",
            "--process-id",
            "902",
            "--account-digest",
            ACCOUNT_DIGEST,
        )
        first, first_result = self.invoke(
            *arguments,
            sensor_case="threshold",
            extra_environment={"FAKE_CODEXBAR_LOG": str(sensor_log)},
        )
        self.assert_result(
            first,
            first_result,
            action="CheckUsage",
            exit_code=3,
            status="blocked",
            code="hive.usage-limited",
        )
        self.assertEqual(first_result["data"]["selected_window"], "session")
        self.assertEqual(len(sensor_log.read_text(encoding="utf-8").splitlines()), 2)
        marker_path = self.consumer / first_result["changed_paths"][0]
        marker = json.loads(marker_path.read_text(encoding="utf-8"))
        self.assertEqual(marker["host_scope"], "codex")
        self.assertEqual(marker["process_id"], 902)
        self.assertEqual(marker["decision"], "halted")
        self.assertEqual(marker["threshold_remaining_percent"], 10)
        self.assertEqual(marker["revision"], 1)
        self.assertNotIn(RAW_ACCOUNT, marker_path.read_text(encoding="utf-8"))

        repeated, repeated_result = self.invoke(
            *arguments,
            sensor_case="allow",
            extra_environment={"FAKE_CODEXBAR_LOG": str(sensor_log)},
        )
        self.assert_result(
            repeated,
            repeated_result,
            action="CheckUsage",
            exit_code=3,
            status="blocked",
            code="hive.usage-limited",
        )
        self.assertEqual(repeated_result["changed_paths"], [])
        self.assertEqual(len(sensor_log.read_text(encoding="utf-8").splitlines()), 2)

    def test_explicit_disable_bypasses_sensor_and_enable_reapplies_latch(self) -> None:
        self.write_halt(session_id="session-bypass", process_id=903)
        disabled, _ = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-bypass",
            "--process-id",
            "903",
            "--action",
            "disable",
            "--confirm-session-disable",
        )
        self.assertEqual(disabled.returncode, 0, disabled.stderr)

        empty_path = self.work_root / "empty-path"
        empty_path.mkdir()
        bypassed, bypassed_result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-bypass",
            "--process-id",
            "903",
            extra_environment={"PATH": str(empty_path)},
        )
        self.assert_result(
            bypassed,
            bypassed_result,
            action="CheckUsage",
            exit_code=0,
            status="success",
            code="hive.usage-session-bypassed",
        )
        self.assert_preflight_only(bypassed_result)

        enabled, _ = self.invoke(
            "usage",
            "session",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-bypass",
            "--process-id",
            "903",
            "--action",
            "enable",
        )
        self.assertEqual(enabled.returncode, 0, enabled.stderr)
        blocked, blocked_result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "session-bypass",
            "--process-id",
            "903",
            sensor_case="allow",
        )
        self.assertEqual(blocked.returncode, 3, blocked.stderr)
        self.assertEqual(blocked_result["code"], "hive.usage-limited")

    def test_enforce_uses_weekly_only_as_fallback_and_supports_unique_account(
        self,
    ) -> None:
        process, result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "weekly-only",
            "--process-id",
            "904",
            sensor_case="weekly-only",
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.usage-allowed")
        self.assertEqual(result["data"]["selected_window"], "weekly")
        self.assert_preflight_only(result)

        duplicate, duplicate_result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "ambiguous-account",
            "--process-id",
            "905",
            sensor_case="duplicate-account",
        )
        self.assertEqual(duplicate.returncode, 3, duplicate.stderr)
        self.assertEqual(duplicate_result["code"], "hive.usage-unknown")
        self.assertEqual(duplicate_result["data"]["selected_window"], "unknown")

    def test_unknown_sensor_state_is_latched_without_raw_payload(self) -> None:
        process, result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "unknown",
            "--process-id",
            "906",
            "--account-digest",
            ACCOUNT_DIGEST,
            sensor_case="malformed",
        )
        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["code"], "hive.usage-unknown")
        marker = (self.consumer / result["changed_paths"][0]).read_text(
            encoding="utf-8"
        )
        self.assertNotIn(RAW_ACCOUNT, marker)
        self.assertNotIn("{not-json", marker)
        self.assertEqual(json.loads(marker)["decision"], "usage-unknown")

    def test_enforce_rejects_malformed_oversized_and_symlink_halt_markers(
        self,
    ) -> None:
        session_id = "hostile-marker"
        digest = hashlib.sha256(b"codex\0" + session_id.encode()).hexdigest()
        marker = (
            self.consumer
            / ".hive/runtime/usage-guard/sessions"
            / digest
            / "halt.json"
        )
        marker.parent.mkdir(parents=True)
        invalid_evidence = (
            json.dumps(
                {
                    "schema_version": 1,
                    "host_scope": "codex",
                    "session_id_digest": "sha256:" + digest,
                    "process_id": 907,
                    "decision": "halted",
                    "selected_window": "session",
                    "threshold_remaining_percent": 10,
                    "measured_at": 1_750_000_000,
                    "evidence_digest": "raw-provider-payload",
                    "revision": 1,
                },
                separators=(",", ":"),
                sort_keys=True,
            )
            + "\n"
        ).encode()
        for payload in (
            b"{not-json\n",
            b"x" * (16 * 1024 + 1),
            invalid_evidence,
        ):
            with self.subTest(size=len(payload)):
                marker.write_bytes(payload)
                process, result = self.invoke(
                    "usage",
                    "enforce",
                    "--target",
                    str(self.consumer),
                    "--session-id",
                    session_id,
                    "--process-id",
                    "907",
                    sensor_case="allow",
                )
                self.assertEqual(process.returncode, 3, process.stderr)
                self.assertEqual(result["code"], "hive.usage-control-blocked")
                self.assertEqual(marker.read_bytes(), payload)

        if os.name != "nt":
            outside = self.work_root / "outside-marker"
            outside.write_bytes(b"user-owned\n")
            marker.unlink()
            marker.symlink_to(outside)
            process, result = self.invoke(
                "usage",
                "enforce",
                "--target",
                str(self.consumer),
                "--session-id",
                session_id,
                "--process-id",
                "907",
                sensor_case="allow",
            )
            self.assertEqual(process.returncode, 3, process.stderr)
            self.assertEqual(result["code"], "hive.usage-control-blocked")
            self.assertEqual(outside.read_bytes(), b"user-owned\n")

    def test_host_scope_changes_session_digest_and_wrong_fallback_provider_fails_closed(
        self,
    ) -> None:
        config = self.consumer / ".hive/config/harness.toml"
        config.write_text(
            HARNESS_CONFIG.replace('primary_host = "codex"', 'primary_host = "claude"'),
            encoding="utf-8",
        )
        sensor_log = self.work_root / "unused-sensor.log"
        process, result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "same-session",
            "--process-id",
            "908",
            sensor_case="allow",
            extra_environment={"FAKE_CODEXBAR_LOG": str(sensor_log)},
        )
        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["code"], "hive.usage-unknown")
        self.assertEqual(result["data"]["host_scope"], "claude")
        expected = "sha256:" + hashlib.sha256(
            b"claude\0same-session"
        ).hexdigest()
        self.assertEqual(result["data"]["session_id_digest"], expected)
        self.assertEqual(
            len(sensor_log.read_text(encoding="utf-8").splitlines()),
            2,
            "missing Claude capture must try the qualified fallback once",
        )

    def test_missing_claude_capture_and_fallback_offer_exact_install_preview(
        self,
    ) -> None:
        config = self.consumer / ".hive/config/harness.toml"
        config.write_text(
            HARNESS_CONFIG.replace(
                'primary_host = "codex"',
                'primary_host = "claude"',
            ),
            encoding="utf-8",
        )
        empty_path = self.work_root / "empty-fallback-path"
        empty_path.mkdir()

        process, result = self.invoke(
            "usage",
            "enforce",
            "--target",
            str(self.consumer),
            "--session-id",
            "missing-fallback",
            "--process-id",
            "909",
            extra_environment={"PATH": str(empty_path)},
        )

        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["code"], "hive.usage-unknown")
        self.assertIn(
            "hive usage fallback-install --host claude --dry-run --output json",
            result["next_action"],
        )
        self.assertNotIn("<host>", result["next_action"])

    @unittest.skipIf(os.name == "nt", "symlink fixture is POSIX-specific")
    def test_threshold_rejects_a_symlinked_owned_config(self) -> None:
        real = self.work_root / "outside.toml"
        real.write_text(HARNESS_CONFIG, encoding="utf-8")
        config = self.consumer / ".hive/config/harness.toml"
        config.unlink()
        config.symlink_to(real)
        before = real.read_bytes()

        process, result = self.invoke(
            "usage",
            "threshold",
            "--target",
            str(self.consumer),
            "--remaining-percent",
            "25",
        )

        self.assert_result(
            process,
            result,
            action="SetUsageThreshold",
            exit_code=3,
            status="blocked",
            code="hive.usage-control-blocked",
        )
        self.assertEqual(real.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
