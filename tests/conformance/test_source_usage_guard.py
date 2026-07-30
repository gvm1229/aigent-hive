#!/usr/bin/env python3
"""Black-box regression and hostile tests for the source-only usage safeguard."""

from __future__ import annotations

import datetime as dt
import json
import os
import runpy
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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
        self.fake_bin = self.root / "bin"
        self.fake_bin.mkdir()
        self.fixture = self.root / "usage.json"
        self.native_fixture = self.root / "native-usage.json"
        self.call_log = self.root / "sensor-calls.log"
        self.write_fake_codex_unsupported()
        self.write_fake_codexbar()
        self.environment = {
            **os.environ,
            "PATH": f"{self.fake_bin}{os.pathsep}{os.environ.get('PATH', '')}",
            "HIVE_SOURCE_USAGE_FIXTURE": str(self.fixture),
            "HIVE_CODEX_NATIVE_FIXTURE": str(self.native_fixture),
            "HIVE_USAGE_SENSOR_CALL_LOG": str(self.call_log),
            "HIVE_FAKE_CODEX_TARGET": str(self.fake_command_path("codex")),
            "HIVE_FAKE_CODEXBAR_TARGET": str(self.fake_bin / "codexbar"),
            "HIVE_USAGE_UNKNOWN_RETRY_DELAY_SECONDS": "0",
            "HIVE_USAGE_TEST_PROCESS_ID": str(os.getpid()),
            "HIVE_USAGE_TEST_PROCESS_START": "fixture-process-start-a",
            "CODEX_THREAD_ID": self.session_id,
        }

    def tearDown(self) -> None:
        try:
            self.run_guard("watch-stop")
        except Exception:
            pass
        self.temporary.cleanup()

    def test_write_json_skips_unavailable_fchmod(self) -> None:
        guard_namespace = runpy.run_path(str(GUARD))
        root = self.root.resolve()
        target = root / "state.json"

        with mock.patch.object(
            guard_namespace["os"],
            "fchmod",
            None,
            create=True,
        ):
            guard_namespace["write_json"](
                root,
                target,
                {"schema_version": 1},
            )

        self.assertEqual(
            json.loads(target.read_text(encoding="utf-8")),
            {"schema_version": 1},
        )
        self.assertEqual(list(root.glob(".state.json.*")), [])

    def test_write_json_closes_descriptor_before_failed_write_cleanup(
        self,
    ) -> None:
        guard_namespace = runpy.run_path(str(GUARD))
        root = self.root.resolve()
        target = root / "state.json"
        captured: dict[str, int | Path] = {}
        original_mkstemp = guard_namespace["tempfile"].mkstemp

        def capture_mkstemp(*args: object, **kwargs: object) -> tuple[int, str]:
            descriptor, temporary = original_mkstemp(*args, **kwargs)
            captured["descriptor"] = descriptor
            captured["temporary"] = Path(temporary)
            return descriptor, temporary

        with (
            mock.patch.object(
                guard_namespace["tempfile"],
                "mkstemp",
                side_effect=capture_mkstemp,
            ),
            mock.patch.object(
                guard_namespace["os"],
                "fchmod",
                side_effect=OSError("permission update failed"),
                create=True,
            ),
        ):
            with self.assertRaisesRegex(OSError, "permission update failed"):
                guard_namespace["write_json"](
                    root,
                    target,
                    {"schema_version": 1},
                )

        descriptor = captured["descriptor"]
        self.assertIsInstance(descriptor, int)
        try:
            with self.assertRaises(OSError):
                os.fstat(descriptor)
        finally:
            try:
                os.close(descriptor)
            except OSError:
                pass
        temporary = captured["temporary"]
        self.assertIsInstance(temporary, Path)
        self.assertFalse(temporary.exists())
        self.assertFalse(target.exists())

    def test_windows_watcher_lease_read_skips_locked_first_byte(self) -> None:
        guard_namespace = runpy.run_path(str(GUARD))
        payload = b'{"schema_version": 1}\n'
        descriptor, temporary = tempfile.mkstemp(dir=self.root.resolve())
        original_read = os.read

        def deny_locked_byte(candidate: int, size: int) -> bytes:
            if os.lseek(candidate, 0, os.SEEK_CUR) == 0:
                raise PermissionError("locked byte")
            return original_read(candidate, size)

        try:
            os.write(descriptor, payload)
            os.lseek(descriptor, 0, os.SEEK_SET)
            with (
                mock.patch.object(guard_namespace["os"], "name", "nt"),
                mock.patch.object(
                    guard_namespace["os"],
                    "read",
                    side_effect=deny_locked_byte,
                ),
            ):
                actual = guard_namespace["read_watcher_lease_payload"](
                    descriptor,
                    len(payload) + 1,
                )
        finally:
            os.close(descriptor)
            Path(temporary).unlink(missing_ok=True)

        self.assertEqual(actual, payload)

    def switch_codex_thread(self, session_id: str) -> None:
        self.session_id = session_id
        self.environment["CODEX_THREAD_ID"] = session_id

    def fake_command_path(self, name: str) -> Path:
        suffix = ".cmd" if os.name == "nt" else ""
        return self.fake_bin / f"{name}{suffix}"

    def fake_script_path(self, name: str) -> Path:
        suffix = ".py" if os.name == "nt" else ""
        return self.fake_bin / f"{name}{suffix}"

    def wrap_fake_python_command(self, name: str, script: Path) -> None:
        if os.name != "nt":
            return
        self.fake_command_path(name).write_text(
            f'@echo off\r\n"{sys.executable}" "{script}" %*\r\n',
            encoding="ascii",
        )

    def write_fake_codexbar(self) -> None:
        executable = self.fake_script_path("codexbar")
        executable.write_text(
            """#!/usr/bin/env python3
import os
import pathlib
import sys

with pathlib.Path(os.environ["HIVE_USAGE_SENSOR_CALL_LOG"]).open("a") as stream:
    stream.write("codexbar " + " ".join(sys.argv[1:]) + "\\n")
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
        self.wrap_fake_python_command("codexbar", executable)

    def hide_codexbar(self) -> None:
        self.fake_command_path("codexbar").unlink(missing_ok=True)
        self.fake_script_path("codexbar").unlink(missing_ok=True)
        if os.name != "nt":
            python = self.fake_bin / "python3"
            if not python.exists():
                python.symlink_to(sys.executable)
        self.environment["PATH"] = str(self.fake_bin)

    def write_fake_codexbar_transient(self) -> None:
        executable = self.fake_script_path("codexbar")
        executable.write_text(
            """#!/usr/bin/env python3
import os
import pathlib
import sys

log = pathlib.Path(os.environ["HIVE_USAGE_SENSOR_CALL_LOG"])
with log.open("a") as stream:
    stream.write("codexbar " + " ".join(sys.argv[1:]) + "\\n")
if sys.argv[1:] == ["--version"]:
    print("CodexBar 0.45.2")
    raise SystemExit(0)
if sys.argv[1:] == [
    "usage", "--provider", "codex", "--all-accounts", "--source", "cli",
    "--format", "json", "--json-only"
]:
    counter = log.with_suffix(".counter")
    attempts = int(counter.read_text()) if counter.exists() else 0
    counter.write_text(str(attempts + 1))
    if attempts == 0:
        print("{")
    else:
        sys.stdout.write(pathlib.Path(os.environ["HIVE_SOURCE_USAGE_FIXTURE"]).read_text())
    raise SystemExit(0)
raise SystemExit(64)
""",
            encoding="utf-8",
        )
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
        self.wrap_fake_python_command("codexbar", executable)

    def write_fake_brew(self, *, version: str = "Homebrew 4.4.20") -> None:
        executable = self.fake_bin / "brew"
        executable.write_text(
            f"""#!{sys.executable}
import os
import pathlib
import stat
import sys

with pathlib.Path(os.environ["HIVE_USAGE_SENSOR_CALL_LOG"]).open("a") as stream:
    stream.write("brew " + " ".join(sys.argv[1:]) + "\\n")
if sys.argv[1:] == ["--version"]:
    print({version!r})
    raise SystemExit(0)
if sys.argv[1:] in (
    ["install", "--cask", "codexbar"],
    ["install", "steipete/tap/codexbar"],
):
    target = pathlib.Path(os.environ["HIVE_FAKE_CODEXBAR_TARGET"])
    target.write_text(
        "#!{sys.executable}\\n"
        "import sys\\n"
        "print('CodexBar 0.45.2') if sys.argv[1:] == ['--version'] "
        "else sys.exit(64)\\n"
    )
    target.chmod(target.stat().st_mode | stat.S_IXUSR)
    raise SystemExit(0)
raise SystemExit(64)
""",
            encoding="utf-8",
        )
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)

    def write_fake_codex_unsupported(self) -> None:
        executable = self.fake_script_path("codex")
        executable.write_text(
            """#!/usr/bin/env python3
import os
import pathlib
import sys

with pathlib.Path(os.environ["HIVE_USAGE_SENSOR_CALL_LOG"]).open("a") as stream:
    stream.write("codex " + " ".join(sys.argv[1:]) + "\\n")
if sys.argv[1:] == ["--version"]:
    print("codex-cli 0.144.5")
    raise SystemExit(0)
raise SystemExit(64)
""",
            encoding="utf-8",
        )
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
        self.wrap_fake_python_command("codex", executable)

    def write_fake_codex_native(self) -> None:
        executable = self.fake_script_path("codex")
        executable.write_text(
            """#!/usr/bin/env python3
import json
import os
import pathlib
import sys

with pathlib.Path(os.environ["HIVE_USAGE_SENSOR_CALL_LOG"]).open("a") as stream:
    stream.write("codex " + " ".join(sys.argv[1:]) + "\\n")
if sys.argv[1:] == ["--version"]:
    print("codex-cli 0.145.0")
    raise SystemExit(0)
if sys.argv[1:] != ["app-server", "--stdio"]:
    raise SystemExit(64)
fixture = json.loads(
    pathlib.Path(os.environ["HIVE_CODEX_NATIVE_FIXTURE"]).read_text()
)
for line in sys.stdin:
    request = json.loads(line)
    request_id = request.get("id")
    method = request.get("method")
    if method == "initialize":
        response = {"id": request_id, "result": {"userAgent": "test"}}
    elif method == "account/read":
        response = {"id": request_id, "result": fixture["account"]}
    elif method == "account/rateLimits/read":
        response = {"id": request_id, "result": fixture["rate_limits"]}
    else:
        continue
    print(json.dumps(response), flush=True)
""",
            encoding="utf-8",
        )
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
        self.wrap_fake_python_command("codex", executable)

    def write_fake_codex_identity_swap(self) -> None:
        executable = self.fake_script_path("codex")
        executable.write_text(
            """#!/usr/bin/env python3
import os
import pathlib
import sys

path = pathlib.Path(os.environ["HIVE_FAKE_CODEX_TARGET"])
with pathlib.Path(os.environ["HIVE_USAGE_SENSOR_CALL_LOG"]).open("a") as stream:
    stream.write("codex " + " ".join(sys.argv[1:]) + "\\n")
if sys.argv[1:] == ["--version"]:
    with path.open("a") as stream:
        stream.write("\\nrem identity changed after qualification\\n")
    print("codex-cli 0.145.0")
    raise SystemExit(0)
raise SystemExit(64)
""",
            encoding="utf-8",
        )
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
        self.wrap_fake_python_command("codex", executable)

    def write_native_usage(
        self,
        *,
        used: float,
        minutes: int = 10080,
        plan: str = "pro",
        limit_id: str = "codex",
    ) -> None:
        resets_at = int(
            (dt.datetime.now(dt.timezone.utc) + dt.timedelta(days=1)).timestamp()
        )
        rate_limit = {
            "limitId": limit_id,
            "planType": plan,
            "primary": {
                "usedPercent": used,
                "windowDurationMins": minutes,
                "resetsAt": resets_at,
            },
            "secondary": None,
        }
        self.native_fixture.write_text(
            json.dumps(
                {
                    "account": {
                        "account": {
                            "type": "chatgpt",
                            "email": "native-account@example.invalid",
                            "planType": plan,
                        },
                        "requiresOpenaiAuth": True,
                    },
                    "rate_limits": {
                        "rateLimits": rate_limit,
                        "rateLimitsByLimitId": {"codex": rate_limit},
                    },
                }
            ),
            encoding="utf-8",
        )

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

    def process_is_alive(self, pid: int) -> bool:
        if os.name == "nt":
            import ctypes
            from ctypes import wintypes

            kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
            kernel32.OpenProcess.argtypes = [
                wintypes.DWORD,
                wintypes.BOOL,
                wintypes.DWORD,
            ]
            kernel32.OpenProcess.restype = wintypes.HANDLE
            kernel32.WaitForSingleObject.argtypes = [
                wintypes.HANDLE,
                wintypes.DWORD,
            ]
            kernel32.WaitForSingleObject.restype = wintypes.DWORD
            kernel32.CloseHandle.argtypes = [wintypes.HANDLE]
            kernel32.CloseHandle.restype = wintypes.BOOL
            handle = kernel32.OpenProcess(0x00100000, False, pid)
            if not handle:
                return False
            try:
                return kernel32.WaitForSingleObject(handle, 0) == 0x00000102
            finally:
                kernel32.CloseHandle(handle)
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return False
        except PermissionError:
            return True
        return True

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

    def test_native_codex_is_primary_and_limited_never_calls_codexbar(self) -> None:
        self.write_fake_codex_native()
        self.write_native_usage(used=90)
        self.write_usage(primary=None, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 10, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["sensor"], "codex-app-server")
        self.assertFalse(result["fallback_used"])
        self.assertEqual(result["window"], "weekly")
        calls = self.call_log.read_text(encoding="utf-8")
        self.assertIn("codex app-server --stdio", calls)
        self.assertNotIn("codexbar ", calls)

    def test_malformed_native_codex_uses_codexbar_once(self) -> None:
        self.write_fake_codex_native()
        self.write_native_usage(used=1, minutes=301)
        self.write_usage(primary=None, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["sensor"], "codexbar")
        self.assertTrue(result["fallback_used"])
        calls = self.call_log.read_text(encoding="utf-8")
        self.assertEqual(calls.count("codexbar --version"), 1)
        self.assertEqual(calls.count("codexbar usage"), 1)

    def test_native_integrity_change_fails_closed_without_codexbar(
        self,
    ) -> None:
        self.write_fake_codex_identity_swap()
        self.write_usage(primary=None, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 11, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_decision"], "usage_unknown")
        self.assertEqual(
            result["reason"],
            "Codex executable changed during qualification",
        )
        self.assertNotIn("fallback_install", result)
        calls = self.call_log.read_text(encoding="utf-8")
        self.assertEqual(calls.count("codex --version"), 1)
        self.assertNotIn("codexbar ", calls)

    def test_source_and_shipping_integrity_failure_contracts_match(
        self,
    ) -> None:
        source = GUARD.read_text(encoding="utf-8")
        shipping = (
            REPOSITORY_ROOT / "crates/hive-cli/src/usage.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("class NativeSensorIntegrity", source)
        self.assertIn(
            "except ("
            "\n        NativeSensorUnavailable,"
            "\n        NativeSensorUnsupported,"
            "\n        NativeSensorMalformed,",
            source,
        )
        self.assertIn(
            "Self::Unavailable | Self::Unsupported | "
            "Self::UnsupportedVersion | Self::Malformed",
            shipping,
        )
        self.assertIn(
            "every_native_integrity_error_fails_closed_without_codexbar",
            shipping,
        )

    def test_missing_fallback_returns_sanitized_provider_install_preview(
        self,
    ) -> None:
        self.hide_codexbar()

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_decision"], "usage_unknown")
        self.assertEqual(result["enforcement_decision"], "allowed")
        self.assertTrue(result["transient_unknown_ignored"])
        self.assertFalse(result["halt_marker"])
        self.assertEqual(result["fallback_install"]["provider"], "codex")
        preview = (
            "python3 .agents/skills/hive-usage-guard/scripts/guard.py "
            "fallback-install --host codex --dry-run --json"
        )
        self.assertEqual(result["next_action"], preview)
        self.assertEqual(
            result["fallback_install"]["command_preview"],
            preview,
        )
        self.assertNotIn(str(self.root), completed.stdout)
        self.assertNotIn("example.invalid", completed.stdout)
        self.assertFalse((self.fake_bin / "brew").exists())

    @unittest.skipUnless(
        sys.platform == "darwin" or sys.platform.startswith("linux"),
        "source fallback install has no qualified adapter on this platform",
    )
    def test_fallback_install_dry_run_qualifies_brew_without_installing(
        self,
    ) -> None:
        self.hide_codexbar()
        self.write_fake_brew()

        completed = self.run_guard(
            "fallback-install",
            "--host",
            "claude",
            "--dry-run",
        )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(
            result["code"],
            "hive.usage-fallback-install-preview",
        )
        self.assertEqual(result["provider"], "claude")
        self.assertEqual(result["package_manager"], "brew")
        expected_preview = (
            "brew install --cask codexbar"
            if sys.platform == "darwin"
            else "brew install steipete/tap/codexbar"
        )
        self.assertEqual(result["command_preview"], expected_preview)
        self.assertEqual(result["consent_scope"], "current-action")
        self.assertFalse(result["credentials_requested"])
        self.assertFalse(result["provider_cli_reinstall"])
        self.assertFalse(result["manual_cookie_requested"])
        self.assertIn("--host claude --apply --confirm-install", result["next_action"])
        self.assertEqual(
            self.call_log.read_text(encoding="utf-8").splitlines(),
            ["brew --version"],
        )
        self.assertFalse((self.fake_bin / "codexbar").exists())

    def test_fallback_install_apply_requires_current_action_confirmation(
        self,
    ) -> None:
        self.hide_codexbar()
        self.write_fake_brew()

        completed = self.run_guard(
            "fallback-install",
            "--host",
            "antigravity",
            "--apply",
        )

        self.assertEqual(completed.returncode, 64, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["status"], "error")
        self.assertEqual(result["code"], "hive.invalid-input")
        self.assertIn("--confirm-install", result["reason"])
        self.assertFalse(self.call_log.exists())
        self.assertFalse((self.fake_bin / "codexbar").exists())

    @unittest.skipUnless(
        sys.platform == "darwin" or sys.platform.startswith("linux"),
        "source fallback install has no qualified adapter on this platform",
    )
    def test_fallback_install_apply_uses_only_fake_qualified_adapter(
        self,
    ) -> None:
        self.hide_codexbar()
        self.write_fake_brew()

        completed = self.run_guard(
            "fallback-install",
            "--host",
            "codex",
            "--apply",
            "--confirm-install",
        )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["code"], "hive.usage-fallback-installed")
        calls = self.call_log.read_text(encoding="utf-8").splitlines()
        self.assertEqual(calls[0], "brew --version")
        self.assertIn(
            calls[1],
            (
                "brew install --cask codexbar",
                "brew install steipete/tap/codexbar",
            ),
        )
        self.assertTrue((self.fake_bin / "codexbar").is_file())

    @unittest.skipUnless(
        sys.platform == "darwin" or sys.platform.startswith("linux"),
        "source fallback install has no qualified adapter on this platform",
    )
    def test_unqualified_package_manager_never_reaches_install(self) -> None:
        self.hide_codexbar()
        self.write_fake_brew(version="untrusted manager")

        completed = self.run_guard(
            "fallback-install",
            "--host",
            "codex",
            "--dry-run",
        )

        self.assertEqual(completed.returncode, 64, completed.stderr)
        result = self.output(completed)
        self.assertEqual(
            result["code"],
            "hive.usage-fallback-install-unsupported",
        )
        self.assertEqual(
            result["reason"],
            "supported package manager brew could not be qualified",
        )
        self.assertEqual(
            self.call_log.read_text(encoding="utf-8").splitlines(),
            ["brew --version"],
        )

    @unittest.skipUnless(
        sys.platform == "darwin" or sys.platform.startswith("linux"),
        "source fallback install has no qualified adapter on this platform",
    )
    def test_fallback_install_package_manager_unavailable_is_stable(
        self,
    ) -> None:
        self.hide_codexbar()

        completed = self.run_guard(
            "fallback-install",
            "--host",
            "codex",
            "--dry-run",
        )

        self.assertEqual(completed.returncode, 64, completed.stderr)
        result = self.output(completed)
        self.assertEqual(
            result,
            {
                "action": "InstallUsageFallback",
                "code": "hive.usage-fallback-install-unsupported",
                "reason": "supported package manager brew is unavailable",
                "schema_version": 1,
                "status": "unsupported",
            },
        )

    def test_declining_missing_fallback_keeps_guard_controls_usable(
        self,
    ) -> None:
        self.hide_codexbar()

        unknown = self.run_guard("check")
        disabled = self.run_guard(
            "session-disable",
            "--confirm-session-disable",
        )

        self.assertEqual(unknown.returncode, 0, unknown.stderr)
        self.assertEqual(
            self.output(unknown)["fallback_install"]["decline_effect"],
            "core-usable-automatic-dispatch-usage-unknown",
        )
        self.assertEqual(disabled.returncode, 0, disabled.stderr)
        self.assertFalse(self.output(disabled)["guard_enabled"])

    def test_session_window_takes_precedence_over_weekly(self) -> None:
        self.write_usage(primary={"usedPercent": 90}, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 10, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["window"], "session")
        self.assertEqual(result["remaining_percent"], 10.0)

    def test_malformed_present_session_window_is_transient_unknown(self) -> None:
        self.write_usage(primary={"usedPercent": "bad"}, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_decision"], "usage_unknown")
        self.assertTrue(result["transient_unknown_ignored"])
        self.assertEqual(result["unknown_retry_count"], 1)
        self.assertFalse(result["halt_marker"])

    def test_missing_primary_field_is_transient_unknown(self) -> None:
        self.write_usage(include_primary=False, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_decision"], "usage_unknown")
        self.assertTrue(result["transient_unknown_ignored"])
        self.assertFalse(result["halt_marker"])

    def test_duplicate_primary_key_is_transient_unknown(self) -> None:
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

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_decision"], "usage_unknown")
        self.assertTrue(result["transient_unknown_ignored"])
        self.assertFalse(result["halt_marker"])

    def test_wrong_window_duration_is_transient_unknown(self) -> None:
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

                self.assertEqual(completed.returncode, 0, completed.stderr)
                self.assertEqual(
                    self.output(completed)["quota_decision"], "usage_unknown"
                )

    def test_transient_unknown_recovers_after_one_retry(self) -> None:
        self.write_fake_codexbar_transient()
        self.write_usage(primary=None, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_decision"], "allowed")
        self.assertTrue(result["transient_unknown_recovered"])
        self.assertEqual(result["unknown_retry_count"], 1)
        calls = self.call_log.read_text(encoding="utf-8")
        self.assertEqual(calls.count("codexbar usage"), 2)

    def test_transient_unknown_preserves_previous_confirmed_halt(self) -> None:
        self.write_usage(primary=None, secondary_used=91)
        limited = self.run_guard("check")
        self.write_usage(primary={"usedPercent": "bad"}, secondary_used=1)

        unknown = self.run_guard("check")

        self.assertEqual(limited.returncode, 10, limited.stderr)
        self.assertEqual(unknown.returncode, 10, unknown.stderr)
        result = self.output(unknown)
        self.assertTrue(result["transient_unknown_ignored"])
        self.assertTrue(result["confirmed_halt_preserved"])
        self.assertTrue(result["halt_marker"])

    def test_integrity_then_transient_unknown_preserves_confirmed_halt(
        self,
    ) -> None:
        self.write_usage(primary=None, secondary_used=91)
        limited = self.run_guard("check")
        self.write_fake_codex_identity_swap()
        integrity = self.run_guard("check")
        self.write_fake_codex_unsupported()
        self.write_usage(primary={"usedPercent": "bad"}, secondary_used=1)

        transient = self.run_guard("check")

        self.assertEqual(limited.returncode, 10, limited.stderr)
        self.assertEqual(integrity.returncode, 11, integrity.stderr)
        integrity_result = self.output(integrity)
        self.assertTrue(integrity_result["confirmed_halt_preserved"])
        self.assertTrue(integrity_result["halt_marker"])
        self.assertEqual(transient.returncode, 10, transient.stderr)
        transient_result = self.output(transient)
        self.assertTrue(transient_result["transient_unknown_ignored"])
        self.assertTrue(transient_result["confirmed_halt_preserved"])
        self.assertTrue(transient_result["halt_marker"])

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

    def test_gate_allows_clean_clone_without_omx_state(self) -> None:
        self.write_usage(primary=None, secondary_used=1)

        completed = self.run_guard("gate")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["session_id"], self.session_id)
        self.assertEqual(result["session_source"], "codex-thread")
        self.assertTrue(result["watcher"]["active"])
        self.assertTrue((self.root / ".agents/work/usage-guard").is_dir())
        self.assertFalse((self.root / ".omx").exists())

    def test_gate_uses_process_identity_when_codex_thread_is_missing(self) -> None:
        self.environment.pop("CODEX_THREAD_ID")
        self.write_usage(primary=None, secondary_used=1)

        checked = self.run_guard("check")
        status = self.run_guard("status")

        self.assertEqual(checked.returncode, 0, checked.stderr)
        self.assertEqual(status.returncode, 0, status.stderr)
        checked_result = self.output(checked)
        status_result = self.output(status)
        self.assertEqual(checked_result["session_source"], "codex-process")
        self.assertEqual(status_result["session_source"], "codex-process")
        self.assertEqual(checked_result["session_id"], status_result["session_id"])

    def test_gate_ignores_invalid_omx_session_state(self) -> None:
        state = self.root / ".omx/state/session.json"
        state.parent.mkdir(parents=True)
        state.write_bytes(b"{not-json\n")
        before = state.read_bytes()
        self.write_usage(primary=None, secondary_used=1)

        completed = self.run_guard("gate")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(self.output(completed)["session_id"], self.session_id)
        self.assertEqual(state.read_bytes(), before)

    def test_session_disable_requires_confirmation(self) -> None:
        self.write_usage(primary=None, secondary_used=91)

        refused = self.run_guard("session-disable")
        disabled = self.run_guard(
            "session-disable", "--confirm-session-disable"
        )
        bypassed = self.run_guard("check")

        self.assertEqual(refused.returncode, 64, refused.stderr)
        self.assertEqual(disabled.returncode, 0, disabled.stderr)
        self.assertEqual(bypassed.returncode, 0, bypassed.stderr)
        bypass_result = self.output(bypassed)
        self.assertEqual(bypass_result["quota_decision"], "not_checked")
        self.assertEqual(bypass_result["enforcement_decision"], "session_bypass")

    def test_session_disable_does_not_transfer_to_new_codex_thread(self) -> None:
        self.write_usage(primary=None, secondary_used=91)
        disabled = self.run_guard(
            "session-disable", "--confirm-session-disable"
        )
        self.switch_codex_thread("22222222-2222-2222-2222-222222222222")

        next_session = self.run_guard("check")

        self.assertEqual(disabled.returncode, 0, disabled.stderr)
        self.assertEqual(next_session.returncode, 10, next_session.stderr)
        self.assertTrue(self.output(next_session)["guard_enabled"])

    def test_gate_retires_watcher_from_previous_codex_thread(self) -> None:
        self.write_usage(primary=None, secondary_used=1)
        first = self.run_guard("gate")
        first_pid = int(self.output(first)["watcher"]["pid"])
        first_state = (
            self.root
            / ".agents/work/usage-guard/sessions"
            / self.session_id
            / "watcher.json"
        )
        self.switch_codex_thread("22222222-2222-2222-2222-222222222222")

        second = self.run_guard("gate")

        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(second.returncode, 0, second.stderr)
        second_pid = int(self.output(second)["watcher"]["pid"])
        self.assertNotEqual(first_pid, second_pid)
        self.assertFalse(self.process_is_alive(first_pid))
        self.assertFalse(first_state.exists())

    def test_process_creation_change_never_reuses_session_bypass(self) -> None:
        disabled = self.run_guard(
            "session-disable", "--confirm-session-disable"
        )
        self.environment["HIVE_USAGE_TEST_PROCESS_START"] = (
            "fixture-process-start-b"
        )
        self.write_usage(primary=None, secondary_used=1)

        completed = self.run_guard("check")

        self.assertEqual(disabled.returncode, 0, disabled.stderr)
        self.assertEqual(completed.returncode, 11, completed.stderr)
        self.assertEqual(self.output(completed)["status"], "usage_unknown")

    def test_watch_stop_never_signals_unrelated_tokenized_process(self) -> None:
        disabled = self.run_guard(
            "session-disable", "--confirm-session-disable"
        )
        self.assertEqual(disabled.returncode, 0, disabled.stderr)
        session_root = (
            self.root
            / ".agents/work/usage-guard/sessions"
            / self.session_id
        )
        control = json.loads(
            (session_root / "control.json").read_text(encoding="utf-8")
        )
        sleeper = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(30)"]
        )
        try:
            guard_namespace = runpy.run_path(str(GUARD))
            watcher = {
                "schema_version": 1,
                "session_id": self.session_id,
                "session_pid": control["session_pid"],
                "session_source": "codex-thread",
                "session_process_start": control["session_process_start"],
                "pid": sleeper.pid,
                "watcher_process_start": guard_namespace[
                    "process_start_digest"
                ](sleeper.pid),
                "watcher_nonce_digest": "sha256:" + "0" * 64,
                "started_at": timestamp(),
                "script": str(GUARD),
            }
            (session_root / "watcher.json").write_text(
                json.dumps(watcher),
                encoding="utf-8",
            )

            completed = self.run_guard("watch-stop")

            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertIsNone(sleeper.poll())
            self.assertFalse((session_root / "watcher.json").exists())
        finally:
            if sleeper.poll() is None:
                sleeper.terminate()
            sleeper.wait(timeout=5)

    def test_disabled_gate_does_not_initialize_quota_sensor(self) -> None:
        disabled = self.run_guard(
            "session-disable", "--confirm-session-disable"
        )
        self.hide_codexbar()
        self.call_log.unlink(missing_ok=True)

        bypassed = self.run_guard("gate")

        self.assertEqual(disabled.returncode, 0, disabled.stderr)
        self.assertEqual(bypassed.returncode, 0, bypassed.stderr)
        result = self.output(bypassed)
        self.assertEqual(result["quota_decision"], "not_checked")
        self.assertEqual(result["enforcement_decision"], "session_bypass")
        self.assertFalse(self.call_log.exists())

    def test_status_does_not_initialize_quota_sensor(self) -> None:
        self.hide_codexbar()
        self.call_log.unlink(missing_ok=True)

        completed = self.run_guard("status")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_check"], "not-run")
        self.assertTrue(result["guard_enabled"])
        self.assertFalse(self.call_log.exists())

    def test_status_reports_fresh_last_quota_without_sensor_call(self) -> None:
        self.write_usage(primary=None, secondary_used=1)
        checked = self.run_guard("check")
        calls_before = self.call_log.read_text(encoding="utf-8")

        completed = self.run_guard("status")

        self.assertEqual(checked.returncode, 0, checked.stderr)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["quota_check"], "last-observation")
        self.assertTrue(result["observation_fresh"])
        self.assertEqual(result["used_percent"], 1.0)
        self.assertEqual(result["remaining_percent"], 99.0)
        self.assertEqual(result["window"], "weekly")
        self.assertIn("measured_at", result)
        self.assertEqual(
            self.call_log.read_text(encoding="utf-8"),
            calls_before,
        )

    def test_session_enable_does_not_initialize_quota_sensor(self) -> None:
        self.assertEqual(
            self.run_guard(
                "session-disable", "--confirm-session-disable"
            ).returncode,
            0,
        )
        self.hide_codexbar()
        self.call_log.unlink(missing_ok=True)

        completed = self.run_guard("session-enable")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertTrue(self.output(completed)["guard_enabled"])
        self.assertFalse(self.call_log.exists())

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

    def test_watch_once_ignores_transient_unknown_without_halt_marker(self) -> None:
        self.write_usage(primary={"usedPercent": "bad"}, secondary_used=1)

        completed = self.run_guard("watch", "--once")

        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = self.output(completed)
        self.assertEqual(result["enforcement_decision"], "allowed")
        self.assertTrue(result["transient_unknown_ignored"])
        halt = (
            self.root
            / ".agents"
            / "work"
            / "usage-guard"
            / "sessions"
            / self.session_id
            / "halt.json"
        )
        self.assertFalse(halt.exists())

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

        self.assertIn("even if the user does not name this Skill", skill)
        self.assertIn("Before routing, answering, planning", skill)
        self.assertIn("bare “continue”/“resume”", skill)
        self.assertIn("For every user turn", directive)
        self.assertIn("Only after exit `0`", directive)
        self.assertIn("including simple answers", directive)
        self.assertIn("allow_implicit_invocation: false", interface)

    def test_source_and_shipping_guidance_match_fallback_consent_contract(
        self,
    ) -> None:
        source_skill = (
            REPOSITORY_ROOT
            / ".agents/skills/hive-usage-guard/SKILL.md"
        ).read_text(encoding="utf-8")
        source_directive = (
            REPOSITORY_ROOT
            / ".agents/directives/07-source-usage-guard.md"
        ).read_text(encoding="utf-8")
        canonical = (
            REPOSITORY_ROOT
            / "harness/skills/hive-usage-guard/SKILL.md"
        ).read_bytes()
        projections = (
            "harness/plugins/aigent-hive/skills/hive-usage-guard/SKILL.md",
            "harness/template/.agents/skills/hive-usage-guard/SKILL.md",
            "harness/template/.claude/skills/hive-usage-guard/SKILL.md",
        )

        for projection in projections:
            self.assertEqual(
                (REPOSITORY_ROOT / projection).read_bytes(),
                canonical,
                projection,
            )
        for content in (
            source_skill,
            source_directive,
            canonical.decode("utf-8"),
        ):
            self.assertIn("current-action", content)
            self.assertIn("silently", content)
            self.assertIn("provider CLI", content)
            self.assertIn("manual-cookie", content)
            self.assertIn("package-manager", content)
        self.assertIn("fallback-install --host", source_skill)
        self.assertIn("--apply --confirm-install", source_skill)
        self.assertIn("bounded transient-unknown continuation", source_directive)
        self.assertIn(
            "automatic dispatch `hive.usage-unknown`",
            canonical.decode("utf-8"),
        )

    def test_korean_guide_documents_actual_session_prompts_and_blocking(self) -> None:
        guide = (
            REPOSITORY_ROOT / "docs" / "guides" / "source-usage-guard.md"
        ).read_text(encoding="utf-8")

        self.assertIn("## 실제 session에서 사용", guide)
        self.assertIn("사용량 가드 중지선을 잔여 10%로 설정하고 켜 줘.", guide)
        self.assertIn("이 session에서 사용량 가드를 우회하고 계속해.", guide)
        self.assertIn("session 우회를 해제하고 사용량 가드를 다시 켜 줘.", guide)
        self.assertIn("`계속해`, `resume`, `끝내 줘`", guide)
        self.assertIn(
            "일반 질문, 계획, Skill, tool, write와 후속 task를",
            guide,
        )
        self.assertIn("우회 승인으로 인정 불가", guide)

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
        try:
            control.symlink_to(external)
        except OSError as error:
            self.skipTest(f"symlink creation is unavailable: {error}")

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


def load_tests(
    loader: unittest.TestLoader,
    tests: unittest.TestSuite,
    pattern: str | None,
) -> unittest.TestSuite:
    if (
        os.name != "nt"
        or os.environ.get("HIVE_WINDOWS_SOURCE_USAGE_GUARD_SUBSET") != "skip"
    ):
        return tests
    return unittest.TestSuite()


if __name__ == "__main__":
    unittest.main()
