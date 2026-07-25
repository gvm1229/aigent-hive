#!/usr/bin/env python3
"""Black-box regression and hostile tests for the source-only usage safeguard."""

from __future__ import annotations

import datetime as dt
import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
GUARD = (
    REPOSITORY_ROOT
    / ".agents"
    / "skills"
    / "hive-usage-guard"
    / "scripts"
    / "guard.py"
)


def timestamp() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")


class SourceUsageGuardTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="hive-source-usage-")
        self.root = Path(self.temporary.name)
        (self.root / "hive-source.json").write_text("{}\n", encoding="utf-8")
        (self.root / "AGENTS.md").write_text("# test\n", encoding="utf-8")
        self.session_id = "11111111-1111-1111-1111-111111111111"
        self.write_session(self.session_id)
        self.fake_bin = self.root / "bin"
        self.fake_bin.mkdir()
        self.fixture = self.root / "usage.json"
        self.write_fake_codexbar()
        self.environment = {
            **os.environ,
            "PATH": f"{self.fake_bin}{os.pathsep}{os.environ.get('PATH', '')}",
            "HIVE_SOURCE_USAGE_FIXTURE": str(self.fixture),
        }

    def tearDown(self) -> None:
        try:
            self.run_guard("watch-stop")
        except Exception:
            pass
        self.temporary.cleanup()

    def write_session(self, session_id: str, *, pid: int | None = None) -> None:
        path = self.root / ".omx" / "state"
        path.mkdir(parents=True, exist_ok=True)
        (path / "session.json").write_text(
            json.dumps(
                {
                    "session_id": session_id,
                    "native_session_id": session_id,
                    "cwd": str(self.root),
                    "pid": os.getpid() if pid is None else pid,
                }
            ),
            encoding="utf-8",
        )

    def write_fake_codexbar(self) -> None:
        executable = self.fake_bin / "codexbar"
        executable.write_text(
            """#!/usr/bin/env python3
import os
import pathlib
import sys

if sys.argv[1:] == ["--version"]:
    print("CodexBar 0.45.2")
    raise SystemExit(0)
if sys.argv[1:] == [
    "usage", "--provider", "codex", "--all-accounts", "--source", "cli",
    "--format", "json", "--json-only"
]:
    sys.stdout.write(pathlib.Path(os.environ["HIVE_SOURCE_USAGE_FIXTURE"]).read_text())
    raise SystemExit(0)
raise SystemExit(64)
""",
            encoding="utf-8",
        )
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)

    def write_usage(
        self,
        *,
        primary: object | None = None,
        secondary_used: float = 91,
        include_primary: bool = True,
    ) -> None:
        usage: dict[str, object] = {
            "secondary": {
                "usedPercent": secondary_used,
                "windowMinutes": 10080,
            },
            "updatedAt": timestamp(),
        }
        if include_primary:
            if isinstance(primary, dict):
                primary = {"windowMinutes": 300, **primary}
            usage["primary"] = primary
        self.fixture.write_text(
            json.dumps(
                [
                    {
                        "provider": "codex",
                        "source": "codex-cli",
                        "account": "redacted-in-source-guard-output",
                        "usage": usage,
                    }
                ]
            ),
            encoding="utf-8",
        )

    def run_guard(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(GUARD),
                "--root",
                str(self.root),
                *arguments,
                "--json",
            ],
            cwd=self.root,
            env=self.environment,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=10,
        )

    def output(self, completed: subprocess.CompletedProcess[str]) -> dict[str, object]:
        self.assertTrue(completed.stdout.strip(), completed.stderr)
        return json.loads(completed.stdout.strip().splitlines()[-1])

    def test_weekly_fallback_halts_at_or_below_inclusive_default(self) -> None:
        self.write_usage(primary=None, secondary_used=90)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 10, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["window"], "weekly")
        self.assertEqual(result["remaining_percent"], 10.0)
        self.assertEqual(result["quota_decision"], "halted")
        halt = (
            self.root
            / ".agents"
            / "work"
            / "usage-guard"
            / "sessions"
            / self.session_id
            / "halt.json"
        )
        self.assertTrue(halt.is_file())

    def test_session_window_takes_precedence_over_weekly(self) -> None:
        self.write_usage(primary={"usedPercent": 90}, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 10, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["window"], "session")
        self.assertEqual(result["remaining_percent"], 10.0)

    def test_malformed_present_session_window_never_falls_back_to_weekly(self) -> None:
        self.write_usage(primary={"usedPercent": "bad"}, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 11, completed.stderr)
        self.assertEqual(self.output(completed)["quota_decision"], "usage_unknown")

    def test_missing_primary_field_is_fail_closed(self) -> None:
        self.write_usage(include_primary=False, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 11, completed.stderr)
        self.assertEqual(self.output(completed)["quota_decision"], "usage_unknown")

    def test_duplicate_primary_key_cannot_erase_a_limited_session(self) -> None:
        self.fixture.write_text(
            (
                '[{"provider":"codex","source":"codex-cli","usage":{'
                f'"updatedAt":"{timestamp()}",'
                '"primary":{"usedPercent":99,"windowMinutes":300},'
                '"primary":null,'
                '"secondary":{"usedPercent":1,"windowMinutes":10080}'
                "}}]"
            ),
            encoding="utf-8",
        )

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 11, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_decision"], "usage_unknown")
        self.assertTrue(result["halt_marker"])

    def test_wrong_session_or_weekly_window_duration_is_fail_closed(self) -> None:
        hostile_windows = (
            {
                "primary": {"usedPercent": 1, "windowMinutes": 301},
                "secondary": {"usedPercent": 1, "windowMinutes": 10080},
            },
            {
                "primary": None,
                "secondary": {"usedPercent": 1, "windowMinutes": 300},
            },
        )
        for usage in hostile_windows:
            with self.subTest(usage=usage):
                self.fixture.write_text(
                    json.dumps(
                        [
                            {
                                "provider": "codex",
                                "source": "codex-cli",
                                "usage": {"updatedAt": timestamp(), **usage},
                            }
                        ]
                    ),
                    encoding="utf-8",
                )

                completed = self.run_guard("check")

                self.assertEqual(completed.returncode, 11, completed.stderr)
                self.assertEqual(
                    self.output(completed)["quota_decision"], "usage_unknown"
                )

    def test_threshold_change_is_persistent_and_range_checked(self) -> None:
        self.write_usage(primary=None, secondary_used=91)

        changed = self.run_guard("set-threshold", "8")
        allowed = self.run_guard("check")
        invalid = self.run_guard("set-threshold", "0")

        self.assertEqual(changed.returncode, 0, changed.stderr)
        self.assertEqual(allowed.returncode, 0, allowed.stderr)
        self.assertEqual(self.output(allowed)["threshold_remaining_percent"], 8)
        self.assertEqual(invalid.returncode, 64, invalid.stderr)
        settings = json.loads(
            (
                self.root
                / ".agents"
                / "work"
                / "usage-guard"
                / "settings.json"
            ).read_text(encoding="utf-8")
        )
        self.assertEqual(settings["threshold_remaining_percent"], 8)

    def test_session_disable_requires_confirmation_and_never_transfers(self) -> None:
        self.write_usage(primary=None, secondary_used=91)

        refused = self.run_guard("session-disable")
        disabled = self.run_guard(
            "session-disable", "--confirm-session-disable"
        )
        bypassed = self.run_guard("check")
        self.write_session("22222222-2222-2222-2222-222222222222")
        next_session = self.run_guard("check")

        self.assertEqual(refused.returncode, 64, refused.stderr)
        self.assertEqual(disabled.returncode, 0, disabled.stderr)
        self.assertEqual(bypassed.returncode, 0, bypassed.stderr)
        bypass_result = self.output(bypassed)
        self.assertEqual(bypass_result["quota_decision"], "halted")
        self.assertEqual(bypass_result["enforcement_decision"], "session_bypass")
        self.assertEqual(next_session.returncode, 10, next_session.stderr)
        self.assertTrue(self.output(next_session)["guard_enabled"])

    def test_reenable_immediately_restores_halt(self) -> None:
        self.write_usage(primary=None, secondary_used=91)
        self.assertEqual(
            self.run_guard(
                "session-disable", "--confirm-session-disable"
            ).returncode,
            0,
        )

        enabled = self.run_guard("session-enable")
        checked = self.run_guard("check")

        self.assertEqual(enabled.returncode, 0, enabled.stderr)
        self.assertEqual(checked.returncode, 10, checked.stderr)
        self.assertEqual(self.output(checked)["enforcement_decision"], "halted")

    def test_stale_session_control_pid_fails_closed(self) -> None:
        self.write_usage(primary=None, secondary_used=1)
        control = (
            self.root
            / ".agents"
            / "work"
            / "usage-guard"
            / "sessions"
            / self.session_id
            / "control.json"
        )
        control.parent.mkdir(parents=True)
        control.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "session_id": self.session_id,
                    "session_pid": os.getpid() + 999999,
                    "guard_enabled": False,
                }
            ),
            encoding="utf-8",
        )

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 11, completed.stderr)
        self.assertEqual(self.output(completed)["status"], "usage_unknown")

    def test_watch_once_writes_halt_marker(self) -> None:
        self.write_usage(primary=None, secondary_used=91)

        completed = self.run_guard("watch", "--once")

        self.assertEqual(completed.returncode, 10, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["enforcement_decision"], "halted")

    def test_watcher_lifecycle_is_session_scoped(self) -> None:
        self.write_usage(primary=None, secondary_used=1)

        started = self.run_guard("watch-start")
        status = self.run_guard("watch-status")
        stopped = self.run_guard("watch-stop")

        self.assertEqual(started.returncode, 0, started.stderr)
        self.assertTrue(self.output(started)["active"])
        self.assertEqual(status.returncode, 0, status.stderr)
        self.assertTrue(self.output(status)["active"])
        self.assertEqual(stopped.returncode, 0, stopped.stderr)
        self.assertTrue(self.output(stopped)["stopped"])

    def test_gate_repeatedly_blocks_every_task_until_current_session_bypass(self) -> None:
        self.write_usage(primary=None, secondary_used=91)

        first_task = self.run_guard("gate")
        second_task = self.run_guard("gate")
        disabled = self.run_guard(
            "session-disable", "--confirm-session-disable"
        )
        bypassed_task = self.run_guard("gate")
        enabled = self.run_guard("session-enable")
        blocked_again = self.run_guard("gate")

        self.assertEqual(first_task.returncode, 10, first_task.stderr)
        first_result = self.output(first_task)
        self.assertTrue(first_result["watcher"]["active"])
        self.assertTrue(first_result["halt_marker"])
        self.assertEqual(second_task.returncode, 10, second_task.stderr)
        self.assertTrue(self.output(second_task)["halt_marker"])
        self.assertEqual(disabled.returncode, 0, disabled.stderr)
        self.assertEqual(bypassed_task.returncode, 0, bypassed_task.stderr)
        self.assertEqual(
            self.output(bypassed_task)["enforcement_decision"],
            "session_bypass",
        )
        self.assertEqual(enabled.returncode, 0, enabled.stderr)
        self.assertEqual(blocked_again.returncode, 10, blocked_again.stderr)
        self.assertTrue(self.output(blocked_again)["halt_marker"])

    def test_skill_and_directive_require_implicit_session_wide_turn_gating(self) -> None:
        skill = (
            REPOSITORY_ROOT
            / ".agents"
            / "skills"
            / "hive-usage-guard"
            / "SKILL.md"
        ).read_text(encoding="utf-8")
        directive = (
            REPOSITORY_ROOT
            / ".agents"
            / "directives"
            / "07-source-usage-guard.md"
        ).read_text(encoding="utf-8")
        interface = (
            REPOSITORY_ROOT
            / ".agents"
            / "skills"
            / "hive-usage-guard"
            / "agents"
            / "openai.yaml"
        ).read_text(encoding="utf-8")

        self.assertIn("the user does not need to name this Skill", skill)
        self.assertIn("Before routing, answering, planning", skill)
        self.assertIn("bare “continue”/“resume”", skill)
        self.assertIn("For every user turn", directive)
        self.assertIn("Only after exit `0`", directive)
        self.assertIn("including simple answers", directive)
        self.assertIn("allow_implicit_invocation: true", interface)

    def test_korean_guide_documents_actual_session_prompts_and_blocking(self) -> None:
        guide = (
            REPOSITORY_ROOT / "docs" / "guides" / "source-usage-guard.md"
        ).read_text(encoding="utf-8")
        readme = (REPOSITORY_ROOT / "README.md").read_text(encoding="utf-8")

        self.assertIn("## 실제 session에서 사용", guide)
        self.assertIn("사용량 가드 중지선을 잔여 10%로 설정하고 켜 줘.", guide)
        self.assertIn("이 session에서 사용량 가드를 우회하고 계속해.", guide)
        self.assertIn("session 우회를 해제하고 사용량 가드를 다시 켜 줘.", guide)
        self.assertIn("`계속해`, `resume`, `끝내 줘`", guide)
        self.assertIn("모든 일반 task를", readme)
        self.assertIn("`계속해`나 `resume`만으로는 우회 추론 없음.", readme)

    def test_symlinked_control_state_is_rejected_without_touching_target(self) -> None:
        self.write_usage(primary=None, secondary_used=1)
        external = self.root / "external.json"
        external.write_text('{"sentinel": true}\n', encoding="utf-8")
        control = (
            self.root
            / ".agents"
            / "work"
            / "usage-guard"
            / "sessions"
            / self.session_id
            / "control.json"
        )
        control.parent.mkdir(parents=True)
        control.symlink_to(external)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 11, completed.stderr)
        self.assertEqual(external.read_text(encoding="utf-8"), '{"sentinel": true}\n')

    def test_account_identity_is_never_persisted_or_returned(self) -> None:
        self.write_usage(primary=None, secondary_used=1)
        secret_account = "private-account@example.invalid"
        payload = json.loads(self.fixture.read_text(encoding="utf-8"))
        payload[0]["account"] = secret_account
        payload[0]["usage"]["accountEmail"] = secret_account
        self.fixture.write_text(json.dumps(payload), encoding="utf-8")

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertNotIn(secret_account, completed.stdout)
        observation = (
            self.root
            / ".agents"
            / "work"
            / "usage-guard"
            / "sessions"
            / self.session_id
            / "observation.json"
        )
        self.assertNotIn(secret_account, observation.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
