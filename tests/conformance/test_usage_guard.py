#!/usr/bin/env python3
"""Hostile black-box conformance for the read-only CodexBar usage guard."""

from __future__ import annotations

import hashlib
import json
import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker, ValidationError

from tests.conformance.phase1_support import (
    ACTION_RESULT_SCHEMA,
    Phase1CliTestCase,
    REPOSITORY_ROOT,
    snapshot_tree,
)


USAGE_SNAPSHOT_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/usage-snapshot.schema.json").read_text(
        encoding="utf-8"
    )
)
RAW_ACCOUNT = "usage-guard@example.invalid"
ACCOUNT_DIGEST = "sha256:" + hashlib.sha256(RAW_ACCOUNT.encode()).hexdigest()
OTHER_ACCOUNT_DIGEST = "sha256:" + hashlib.sha256(b"other-account").hexdigest()
CODEXBAR_FIXTURE = (
    REPOSITORY_ROOT / "tests/fixtures/phase5/usage/codexbar_fixture.py"
).read_text(encoding="utf-8")


def valid_snapshot() -> dict[str, object]:
    return {
        "schema_version": 1,
        "sensor_id": "codexbar",
        "sensor_version": "0.45.2",
        "host_scope": "codex",
        "account_scope_digest": ACCOUNT_DIGEST,
        "quota_window": "session",
        "remaining_percent": 57,
        "measured_at_unix_seconds": 1_750_000_000,
        "expires_at_unix_seconds": 1_750_000_060,
        "resets_at_unix_seconds": 1_750_018_000,
        "source_confidence": "high",
    }


class UsageSnapshotSchemaConformance(unittest.TestCase):
    def test_normalized_snapshot_satisfies_the_machine_schema(self) -> None:
        Draft202012Validator.check_schema(USAGE_SNAPSHOT_SCHEMA)

        Draft202012Validator(
            USAGE_SNAPSHOT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(valid_snapshot())

    def test_snapshot_accepts_zero_and_one_hundred_percent_bounds(self) -> None:
        validator = Draft202012Validator(USAGE_SNAPSHOT_SCHEMA)
        for remaining_percent in (0, 100):
            with self.subTest(remaining_percent=remaining_percent):
                instance = valid_snapshot()
                instance["remaining_percent"] = remaining_percent
                validator.validate(instance)

    def test_snapshot_rejects_non_sha256_account_scope_digest(self) -> None:
        instance = valid_snapshot()
        instance["account_scope_digest"] = RAW_ACCOUNT

        with self.assertRaises(ValidationError):
            Draft202012Validator(USAGE_SNAPSHOT_SCHEMA).validate(instance)

    def test_snapshot_rejects_unknown_quota_window(self) -> None:
        instance = valid_snapshot()
        instance["quota_window"] = "monthly"

        with self.assertRaises(ValidationError):
            Draft202012Validator(USAGE_SNAPSHOT_SCHEMA).validate(instance)

    def test_snapshot_rejects_missing_normalized_field(self) -> None:
        instance = valid_snapshot()
        del instance["expires_at_unix_seconds"]

        with self.assertRaises(ValidationError):
            Draft202012Validator(USAGE_SNAPSHOT_SCHEMA).validate(instance)

    def test_snapshot_rejects_negative_unix_seconds(self) -> None:
        instance = valid_snapshot()
        instance["measured_at_unix_seconds"] = -1

        with self.assertRaises(ValidationError):
            Draft202012Validator(USAGE_SNAPSHOT_SCHEMA).validate(instance)

    def test_snapshot_rejects_unknown_source_confidence(self) -> None:
        instance = valid_snapshot()
        instance["source_confidence"] = "unverified"

        with self.assertRaises(ValidationError):
            Draft202012Validator(USAGE_SNAPSHOT_SCHEMA).validate(instance)

    def test_snapshot_rejects_additional_properties(self) -> None:
        instance = valid_snapshot()
        instance["raw_account"] = RAW_ACCOUNT

        with self.assertRaises(ValidationError):
            Draft202012Validator(USAGE_SNAPSHOT_SCHEMA).validate(instance)


class UsageGuardCliConformance(Phase1CliTestCase):
    def setUp(self) -> None:
        super().setUp()
        self.consumer = self.work_root / "consumer"
        self.consumer.mkdir()
        (self.consumer / "sentinel.bin").write_bytes(b"user-owned\n")
        self.fake_bin = self.work_root / "fake-bin"
        self.fake_bin.mkdir()
        self._write_fake_codexbar()

    def _write_fake_codexbar(self) -> None:
        if os.name == "nt":
            script = self.fake_bin / "codexbar.cmd"
            python = subprocess.list2cmdline([os.sys.executable])
            script.write_text(
                f"@{python} \"%~dp0\\codexbar.py\" %*\r\n",
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

    def invoke_usage(
        self,
        case: str,
        *,
        account_digest: str = ACCOUNT_DIGEST,
        threshold: int | None = 10,
        sensor_path: Path | None = None,
        extra_environment: dict[str, str] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        command = [
            str(self.hive_binary),
            "usage",
            "check",
            "--account-digest",
            account_digest,
        ]
        if threshold is not None:
            command.extend(["--threshold", str(threshold)])
        command.extend(["--output", "json"])
        environment = {
            **os.environ,
            "PATH": str(sensor_path or self.fake_bin)
            + os.pathsep
            + os.environ.get("PATH", ""),
            "FAKE_CODEXBAR_CASE": case,
            **(extra_environment or {}),
        }
        process = subprocess.run(
            command,
            cwd=self.consumer,
            check=False,
            text=True,
            capture_output=True,
            timeout=3,
            env=environment,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"usage stdout must be exactly one JSON object: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        self.assertIsInstance(result, dict)
        return process, result

    def assert_usage_result(
        self,
        process: subprocess.CompletedProcess[str],
        result: dict[str, object],
        *,
        exit_code: int,
        status: str,
        code: str,
    ) -> None:
        self.assertEqual(process.returncode, exit_code, process.stderr)
        self.assertEqual(result.get("schema_version"), 1)
        self.assertEqual(result.get("action"), "CheckUsage")
        self.assertEqual(result.get("status"), status)
        self.assertEqual(result.get("exit_code"), exit_code)
        self.assertEqual(result.get("code"), code)
        self.assertEqual(result.get("changed_paths"), [])
        self.assertNotIn(RAW_ACCOUNT, process.stdout)
        self.assertNotIn(RAW_ACCOUNT, process.stderr)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)

    def test_fifty_seven_percent_in_both_windows_allows_dispatch(self) -> None:
        process, result = self.invoke_usage("allow")

        self.assert_usage_result(
            process,
            result,
            exit_code=0,
            status="success",
            code="hive.usage-allowed",
        )

    def test_exactly_ten_percent_blocks_dispatch(self) -> None:
        process, result = self.invoke_usage("threshold")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-limited",
        )

    def test_one_window_below_threshold_blocks_dispatch(self) -> None:
        process, result = self.invoke_usage("one-window-low")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-limited",
        )

    def test_weekly_is_the_fallback_when_session_is_unavailable(self) -> None:
        process, result = self.invoke_usage("weekly-only")

        self.assert_usage_result(
            process,
            result,
            exit_code=0,
            status="success",
            code="hive.usage-allowed",
        )

    def test_weekly_fallback_blocks_at_the_exact_threshold(self) -> None:
        process, result = self.invoke_usage("weekly-only-threshold")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-limited",
        )

    def test_session_takes_precedence_over_a_lower_weekly_window(self) -> None:
        process, result = self.invoke_usage("weekly-low-session-high")

        self.assert_usage_result(
            process,
            result,
            exit_code=0,
            status="success",
            code="hive.usage-allowed",
        )

    def test_session_takes_precedence_over_malformed_or_duplicate_weekly(self) -> None:
        for case in (
            "weekly-malformed-session-high",
            "weekly-duplicate-session-high",
        ):
            with self.subTest(case=case):
                process, result = self.invoke_usage(case)

                self.assert_usage_result(
                    process,
                    result,
                    exit_code=0,
                    status="success",
                    code="hive.usage-allowed",
                )

    def test_session_threshold_blocks_even_when_weekly_is_malformed(self) -> None:
        process, result = self.invoke_usage("session-low-weekly-malformed")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-limited",
        )

    def test_duplicate_weekly_is_unknown_only_when_weekly_is_the_fallback(self) -> None:
        process, result = self.invoke_usage("weekly-duplicate")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_default_threshold_is_ten_percent(self) -> None:
        process, result = self.invoke_usage("threshold", threshold=None)

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-limited",
        )

    def test_missing_codexbar_is_usage_unknown(self) -> None:
        empty_path = self.work_root / "empty-path"
        empty_path.mkdir()

        process, result = self.invoke_usage(
            "allow",
            sensor_path=empty_path,
            extra_environment={"PATH": str(empty_path)},
        )

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_unsupported_codexbar_version_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage("unsupported-version")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_codexbar_timeout_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage(
            "timeout",
            extra_environment={"HIVE_USAGE_TEST_TIMEOUT_MS": "50"},
        )

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_bounded_output_limits_fail_closed(self) -> None:
        for case in ("oversized-version", "oversized-usage", "oversized-stderr"):
            with self.subTest(case=case):
                process, result = self.invoke_usage(case)

                self.assert_usage_result(
                    process,
                    result,
                    exit_code=3,
                    status="blocked",
                    code="hive.usage-unknown",
                )

    def test_malformed_codexbar_json_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage("malformed")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_codexbar_process_error_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage("process-error")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_codexbar_reported_error_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage("sensor-error")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_wrong_account_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage(
            "allow",
            account_digest=OTHER_ACCOUNT_DIGEST,
        )

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_duplicate_account_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage("duplicate-account")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_provider_source_and_identity_mismatch_are_usage_unknown(self) -> None:
        for case in ("wrong-provider", "wrong-source", "missing-identity"):
            with self.subTest(case=case):
                process, result = self.invoke_usage(case)

                self.assert_usage_result(
                    process,
                    result,
                    exit_code=3,
                    status="blocked",
                    code="hive.usage-unknown",
                )

    def test_missing_quota_window_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage("missing-window")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_stale_sample_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage("stale")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    def test_future_sample_is_usage_unknown(self) -> None:
        process, result = self.invoke_usage("future")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    @unittest.skipIf(os.name == "nt", "Unix executable identity fixture")
    def test_executable_change_after_version_fails_closed(self) -> None:
        process, result = self.invoke_usage("executable-change")

        self.assert_usage_result(
            process,
            result,
            exit_code=3,
            status="blocked",
            code="hive.usage-unknown",
        )

    @unittest.skipIf(os.name == "nt", "Unix symlink identity fixture")
    def test_symlink_swap_cannot_change_the_qualified_executable(self) -> None:
        linked_bin = self.work_root / "linked-bin"
        linked_bin.mkdir()
        installed = linked_bin / "CodexBarCLI"
        installed.write_text(
            f"#!{os.sys.executable}\n{CODEXBAR_FIXTURE}",
            encoding="utf-8",
        )
        installed.chmod(installed.stat().st_mode | stat.S_IXUSR)
        malicious = linked_bin / "malicious"
        malicious.write_text("#!/bin/sh\nexit 97\n", encoding="utf-8")
        malicious.chmod(malicious.stat().st_mode | stat.S_IXUSR)
        linked = linked_bin / "codexbar"
        linked.symlink_to(installed)

        process, result = self.invoke_usage(
            "symlink-swap",
            sensor_path=linked_bin,
            extra_environment={
                "FAKE_CODEXBAR_LINK": str(linked),
                "FAKE_CODEXBAR_MALICIOUS": str(malicious),
            },
        )

        self.assert_usage_result(
            process,
            result,
            exit_code=0,
            status="success",
            code="hive.usage-allowed",
        )

    def test_usage_check_performs_no_project_writes(self) -> None:
        for case in ("allow", "threshold", "malformed"):
            with self.subTest(case=case):
                before = snapshot_tree(self.consumer)
                self.invoke_usage(case)
                self.assertEqual(snapshot_tree(self.consumer), before)

    def test_usage_action_result_satisfies_the_machine_schema(self) -> None:
        _, result = self.invoke_usage("allow")

        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)


if __name__ == "__main__":
    unittest.main()
