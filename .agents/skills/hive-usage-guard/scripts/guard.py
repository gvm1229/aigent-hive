#!/usr/bin/env python3
"""Session-bound source-development usage guard with native-first quota sensing."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import math
import os
import queue
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
CODEXBAR_VERSION = "0.45.2"
CODEX_MINIMUM_VERSION = (0, 145, 0)
CODEX_NATIVE_SENSOR = "codex-app-server"
CODEX_APP_SERVER_TIMEOUT_SECONDS = 15
FALLBACK_INSTALL_TIMEOUT_SECONDS = 300
FALLBACK_HOSTS = ("codex", "claude", "antigravity")
SUPPORTED_CODEX_PLANS = {
    "free",
    "go",
    "plus",
    "pro",
    "prolite",
    "team",
    "self_serve_business_usage_based",
    "business",
    "enterprise_cbp_usage_based",
    "enterprise",
    "edu",
}
DEFAULT_THRESHOLD = 10
DEFAULT_POLL_SECONDS = 15
UNKNOWN_RETRY_COUNT = 1
UNKNOWN_RETRY_DELAY_SECONDS = 3.0
MAX_OUTPUT_BYTES = 1024 * 1024
MAX_SNAPSHOT_AGE_SECONDS = 120
EXIT_ALLOWED = 0
EXIT_HALTED = 10
EXIT_UNKNOWN = 11
EXIT_USAGE = 64
USAGE_ARGUMENTS = (
    "usage",
    "--provider",
    "codex",
    "--all-accounts",
    "--source",
    "cli",
    "--format",
    "json",
    "--json-only",
)


class GuardError(RuntimeError):
    """A fail-closed source guard error."""


class SessionExpired(GuardError):
    """The requested session is no longer the current source session."""


class CodexBarUnavailable(GuardError):
    """The optional fallback executable is not installed."""


class FallbackInstallUnsupported(GuardError):
    """No qualified package-manager adapter is available."""


class FallbackInstallInvalid(GuardError):
    """The fallback install action lacks valid current-action consent."""


class NativeSensorError(GuardError):
    """A classified Codex native sensor error."""


class NativeSensorUnavailable(NativeSensorError):
    """The qualified native sensor is not available."""


class NativeSensorUnsupported(NativeSensorError):
    """The native sensor version or account type is unsupported."""


class NativeSensorMalformed(NativeSensorError):
    """The native sensor returned malformed or inconsistent data."""


class NativeSensorIntegrity(NativeSensorError):
    """The native executable identity changed after qualification."""


class DuplicateJsonKey(ValueError):
    """Sensor JSON contained an ambiguous duplicate object key."""


def utc_now() -> dt.datetime:
    return dt.datetime.now(dt.timezone.utc)


def iso_now() -> str:
    return utc_now().isoformat().replace("+00:00", "Z")


def parse_timestamp(raw: Any) -> dt.datetime:
    if not isinstance(raw, str) or not raw.strip():
        raise GuardError("CodexBar omitted usage.updatedAt")
    value = raw.strip().replace("Z", "+00:00")
    try:
        parsed = dt.datetime.fromisoformat(value)
    except ValueError as error:
        raise GuardError("CodexBar returned an invalid usage.updatedAt") from error
    if parsed.tzinfo is None:
        raise GuardError("CodexBar usage.updatedAt lacks a timezone")
    return parsed.astimezone(dt.timezone.utc)


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise DuplicateJsonKey(key)
        value[key] = item
    return value


def discover_root(explicit: str | None) -> Path:
    candidates: list[Path] = []
    if explicit:
        candidates.append(Path(explicit))
    else:
        candidates.extend((Path.cwd(), *Path.cwd().parents))
        script = Path(__file__).resolve()
        candidates.extend((script.parent, *script.parents))
    seen: set[Path] = set()
    for candidate in candidates:
        resolved = candidate.resolve()
        if resolved in seen:
            continue
        seen.add(resolved)
        if (resolved / "hive-source.json").is_file() and (resolved / "AGENTS.md").is_file():
            return resolved
    raise GuardError("Aigent Hive source root was not found")


def assert_safe_parent(root: Path, path: Path) -> None:
    resolved_root = root.resolve()
    try:
        relative = path.relative_to(resolved_root)
    except ValueError as error:
        raise GuardError("usage-guard path escaped the source root") from error
    current = resolved_root
    for part in relative.parts[:-1]:
        current /= part
        if current.exists():
            mode = current.lstat().st_mode
            if stat.S_ISLNK(mode) or not stat.S_ISDIR(mode):
                raise GuardError(f"unsafe usage-guard directory: {current}")
        else:
            current.mkdir(mode=0o700)


def read_json(path: Path, *, required: bool = False) -> dict[str, Any] | None:
    if not path.exists():
        if required:
            raise GuardError(f"required state is missing: {path}")
        return None
    if path.is_symlink():
        raise GuardError(f"refusing symlink state: {path}")
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise GuardError(f"cannot open state: {path}") from error
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > 64 * 1024:
            raise GuardError(f"invalid state file: {path}")
        payload = os.read(descriptor, metadata.st_size + 1)
    finally:
        os.close(descriptor)
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise GuardError(f"malformed state file: {path}") from error
    if not isinstance(value, dict):
        raise GuardError(f"state must be an object: {path}")
    return value


def write_json(root: Path, path: Path, value: dict[str, Any]) -> None:
    assert_safe_parent(root, path)
    if path.exists() and path.is_symlink():
        raise GuardError(f"refusing symlink state: {path}")
    payload = (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary_path = Path(temporary)
    try:
        os.fchmod(descriptor, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_path, path)
    finally:
        temporary_path.unlink(missing_ok=True)


def remove_owned_file(root: Path, path: Path) -> None:
    assert_safe_parent(root, path)
    if not path.exists():
        return
    if path.is_symlink() or not stat.S_ISREG(path.lstat().st_mode):
        raise GuardError(f"refusing non-file owned state: {path}")
    path.unlink()


def state_root(root: Path) -> Path:
    return root / ".agents" / "work" / "usage-guard"


def settings_path(root: Path) -> Path:
    return state_root(root) / "settings.json"


def session_dir(root: Path, session_id: str) -> Path:
    if not session_id or any(character not in "0123456789abcdef-" for character in session_id):
        raise GuardError("invalid Codex session identifier")
    return state_root(root) / "sessions" / session_id


def load_settings(root: Path) -> dict[str, int]:
    value = read_json(settings_path(root))
    if value is None:
        return {
            "threshold_remaining_percent": DEFAULT_THRESHOLD,
            "poll_seconds": DEFAULT_POLL_SECONDS,
        }
    if value.get("schema_version") != SCHEMA_VERSION:
        raise GuardError("unsupported usage-guard settings schema")
    threshold = value.get("threshold_remaining_percent")
    poll_seconds = value.get("poll_seconds", DEFAULT_POLL_SECONDS)
    if (
        isinstance(threshold, bool)
        or not isinstance(threshold, int)
        or not 1 <= threshold <= 99
    ):
        raise GuardError("invalid source usage-guard threshold")
    if (
        isinstance(poll_seconds, bool)
        or not isinstance(poll_seconds, int)
        or not 5 <= poll_seconds <= 300
    ):
        raise GuardError("invalid source usage-guard poll interval")
    return {
        "threshold_remaining_percent": threshold,
        "poll_seconds": poll_seconds,
    }


def save_threshold(root: Path, threshold: int) -> dict[str, Any]:
    if not 1 <= threshold <= 99:
        raise GuardError("threshold must be an integer from 1 through 99")
    current = load_settings(root)
    value = {
        "schema_version": SCHEMA_VERSION,
        "threshold_remaining_percent": threshold,
        "poll_seconds": current["poll_seconds"],
        "updated_at": iso_now(),
    }
    write_json(root, settings_path(root), value)
    return value


def process_is_alive(pid: int) -> bool:
    if pid <= 0:
        return False
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def load_current_session(root: Path, expected_id: str | None = None) -> dict[str, Any]:
    value = read_json(root / ".omx" / "state" / "session.json", required=True)
    assert value is not None
    session_id = value.get("session_id")
    session_pid = value.get("pid")
    session_cwd = value.get("cwd")
    if not isinstance(session_id, str):
        raise GuardError("OMX session state omitted session_id")
    session_dir(root, session_id)
    if expected_id is not None and session_id != expected_id:
        raise SessionExpired("the guarded Codex session is no longer current")
    if isinstance(session_pid, bool) or not isinstance(session_pid, int):
        raise GuardError("OMX session state omitted pid")
    if not process_is_alive(session_pid):
        raise SessionExpired("the guarded Codex session process is no longer active")
    if not isinstance(session_cwd, str) or Path(session_cwd).resolve() != root:
        raise GuardError("OMX session state belongs to another source root")
    return {"session_id": session_id, "pid": session_pid, "cwd": session_cwd}


def control_path(root: Path, session_id: str) -> Path:
    return session_dir(root, session_id) / "control.json"


def observation_path(root: Path, session_id: str) -> Path:
    return session_dir(root, session_id) / "observation.json"


def halt_path(root: Path, session_id: str) -> Path:
    return session_dir(root, session_id) / "halt.json"


def watcher_path(root: Path, session_id: str) -> Path:
    return session_dir(root, session_id) / "watcher.json"


def watcher_log_path(root: Path, session_id: str) -> Path:
    return session_dir(root, session_id) / "watcher.log"


def guard_enabled(root: Path, session: dict[str, Any]) -> bool:
    value = read_json(control_path(root, session["session_id"]))
    if value is None:
        return True
    if (
        value.get("schema_version") != SCHEMA_VERSION
        or value.get("session_id") != session["session_id"]
        or value.get("session_pid") != session["pid"]
        or not isinstance(value.get("guard_enabled"), bool)
    ):
        raise GuardError("invalid or stale session guard control")
    return bool(value["guard_enabled"])


def set_session_enabled(
    root: Path, session: dict[str, Any], enabled: bool, *, confirmed: bool
) -> dict[str, Any]:
    if not enabled and not confirmed:
        raise GuardError(
            "disabling the guard requires explicit user intent and "
            "--confirm-session-disable"
        )
    value = {
        "schema_version": SCHEMA_VERSION,
        "session_id": session["session_id"],
        "session_pid": session["pid"],
        "guard_enabled": enabled,
        "updated_at": iso_now(),
        "scope": "current-session-only",
        "authorization": "explicit-user-intent" if not enabled else "default-restored",
    }
    write_json(root, control_path(root, session["session_id"]), value)
    if not enabled:
        remove_owned_file(root, halt_path(root, session["session_id"]))
    return value


def run_bounded(
    executable: Path,
    arguments: tuple[str, ...],
    timeout: int,
    *,
    sensor_name: str,
    environment: dict[str, str] | None = None,
) -> str:
    try:
        completed = subprocess.run(
            [str(executable), *arguments],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=timeout,
            env=environment,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GuardError(f"{sensor_name} command failed or timed out") from error
    if len(completed.stdout) > MAX_OUTPUT_BYTES or len(completed.stderr) > MAX_OUTPUT_BYTES:
        raise GuardError(f"{sensor_name} output exceeded the source guard limit")
    if completed.returncode != 0:
        raise GuardError(f"{sensor_name} returned a non-zero status")
    try:
        return completed.stdout.decode("utf-8")
    except UnicodeDecodeError as error:
        raise GuardError(f"{sensor_name} output was not UTF-8") from error


def executable_identity(executable: Path) -> tuple[int, int, int, int, int]:
    metadata = executable.stat()
    return (
        metadata.st_dev,
        metadata.st_ino,
        metadata.st_mode,
        metadata.st_size,
        metadata.st_mtime_ns,
    )


def qualify_codexbar() -> tuple[Path, tuple[int, int, int, int, int]]:
    candidate = shutil.which("codexbar")
    if candidate is None:
        raise CodexBarUnavailable("CodexBar is unavailable")
    executable = Path(candidate).resolve()
    if not executable.is_file() or not os.access(executable, os.X_OK):
        raise GuardError("CodexBar executable is invalid")
    identity = executable_identity(executable)
    version = run_bounded(
        executable,
        ("--version",),
        5,
        sensor_name="CodexBar",
    ).strip()
    if executable_identity(executable) != identity:
        raise GuardError("CodexBar executable changed during qualification")
    if version != f"CodexBar {CODEXBAR_VERSION}":
        raise GuardError("CodexBar version is unsupported")
    return executable, identity


def fallback_install_command() -> tuple[str, tuple[str, ...]]:
    if sys.platform == "darwin":
        return "brew", ("install", "--cask", "codexbar")
    if sys.platform.startswith("linux"):
        return "brew", ("install", "steipete/tap/codexbar")
    raise FallbackInstallUnsupported(
        "CodexBar fallback installation is unsupported on this platform"
    )


def qualify_package_manager(
    manager: str,
) -> tuple[Path, tuple[int, int, int, int, int]]:
    if manager != "brew":
        raise FallbackInstallUnsupported(
            "CodexBar fallback installation has no qualified package-manager adapter"
        )
    candidate = shutil.which(manager)
    if candidate is None:
        raise FallbackInstallUnsupported(
            "supported package manager brew is unavailable"
        )
    executable = Path(candidate).resolve()
    if not executable.is_file() or not os.access(executable, os.X_OK):
        raise FallbackInstallUnsupported(
            "supported package manager brew is unavailable"
        )
    identity = executable_identity(executable)
    try:
        version = run_bounded(
            executable,
            ("--version",),
            5,
            sensor_name="Homebrew",
        ).splitlines()
    except GuardError as error:
        raise FallbackInstallUnsupported(
            "supported package manager brew could not be qualified"
        ) from error
    if (
        not version
        or re.fullmatch(r"Homebrew \d+\.\d+\.\d+", version[0].strip()) is None
        or executable_identity(executable) != identity
    ):
        raise FallbackInstallUnsupported(
            "supported package manager brew could not be qualified"
        )
    return executable, identity


def source_guard_command(host: str, *, apply: bool) -> str:
    mode = "--apply --confirm-install" if apply else "--dry-run"
    return (
        "python3 .agents/skills/hive-usage-guard/scripts/guard.py "
        f"fallback-install --host {host} {mode} --json"
    )


def fallback_install(
    host: str,
    *,
    apply: bool,
    confirmed: bool,
) -> dict[str, Any]:
    if host not in FALLBACK_HOSTS:
        raise FallbackInstallInvalid(
            "fallback install host must be codex, claude, or antigravity"
        )
    if apply and not confirmed:
        raise FallbackInstallInvalid(
            "CodexBar installation requires explicit current-action consent "
            "via --confirm-install"
        )
    if not apply and confirmed:
        raise FallbackInstallInvalid(
            "--confirm-install is valid only with --apply"
        )
    manager, install_arguments = fallback_install_command()
    executable, identity = qualify_package_manager(manager)
    preview = " ".join((manager, *install_arguments))
    result: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "action": "InstallUsageFallback",
        "status": "success",
        "code": (
            "hive.usage-fallback-installed"
            if apply
            else "hive.usage-fallback-install-preview"
        ),
        "provider": host,
        "fallback": "codexbar",
        "package_manager": manager,
        "consent_scope": "current-action",
        "credentials_requested": False,
        "provider_cli_reinstall": False,
        "manual_cookie_requested": False,
        "changed_paths": [],
        "command_preview": preview,
        "command_digest": "sha256:"
        + hashlib.sha256(preview.encode("utf-8")).hexdigest(),
    }
    if not apply:
        result["message"] = (
            f"CodexBar fallback installation preview prepared for {host}"
        )
        result["next_action"] = source_guard_command(host, apply=True)
        return result
    if executable_identity(executable) != identity:
        raise FallbackInstallUnsupported(
            "supported package manager brew changed before installation"
        )
    environment = os.environ.copy()
    environment.update(
        {
            "CI": "1",
            "HOMEBREW_NO_AUTO_UPDATE": "1",
            "HOMEBREW_NO_ENV_HINTS": "1",
        }
    )
    run_bounded(
        executable,
        install_arguments,
        FALLBACK_INSTALL_TIMEOUT_SECONDS,
        sensor_name="Homebrew",
        environment=environment,
    )
    if executable_identity(executable) != identity:
        raise FallbackInstallUnsupported(
            "supported package manager brew changed during installation"
        )
    try:
        qualify_codexbar()
    except GuardError as error:
        raise GuardError(
            "CodexBar executable was not available after installation"
        ) from error
    result["message"] = (
        "CodexBar fallback installed after explicit one-action consent"
    )
    result["next_action"] = None
    return result


def verify_native_identity(
    executable: Path,
    expected: tuple[int, int, int, int, int],
    phase: str,
) -> None:
    try:
        current = executable_identity(executable)
    except OSError as error:
        raise NativeSensorIntegrity(
            f"Codex executable changed {phase}"
        ) from error
    if current != expected:
        raise NativeSensorIntegrity(f"Codex executable changed {phase}")


def qualify_codex() -> tuple[Path, tuple[int, int, int, int, int], str]:
    candidate = shutil.which("codex")
    if candidate is None:
        raise NativeSensorUnavailable("Codex native sensor is unavailable")
    executable = Path(candidate).resolve()
    if not executable.is_file() or not os.access(executable, os.X_OK):
        raise NativeSensorIntegrity("Codex native sensor executable is invalid")
    try:
        identity = executable_identity(executable)
    except OSError as error:
        raise NativeSensorIntegrity(
            "Codex executable changed during qualification"
        ) from error
    try:
        version_output = run_bounded(
            executable,
            ("--version",),
            5,
            sensor_name="Codex",
        ).strip()
    except GuardError as error:
        raise NativeSensorError(
            "Codex native sensor version qualification failed"
        ) from error
    match = re.fullmatch(r"codex-cli (\d+)\.(\d+)\.(\d+)", version_output)
    if match is None:
        raise NativeSensorUnsupported(
            "Codex native sensor version is unsupported"
        )
    version = tuple(int(part) for part in match.groups())
    if version < CODEX_MINIMUM_VERSION:
        raise NativeSensorUnsupported(
            "Codex native sensor version is unsupported"
        )
    verify_native_identity(executable, identity, "during qualification")
    return executable, identity, ".".join(str(part) for part in version)


def _jsonl_reader(
    stream: Any,
    messages: queue.Queue[tuple[str, Any]],
) -> None:
    retained = 0
    try:
        while True:
            line = stream.readline(MAX_OUTPUT_BYTES + 1)
            if not line:
                messages.put(("eof", None))
                return
            retained += len(line)
            if retained > MAX_OUTPUT_BYTES:
                messages.put(
                    (
                        "error",
                        GuardError("Codex app-server output exceeded the source guard limit"),
                    )
                )
                return
            try:
                value = json.loads(
                    line.decode("utf-8"),
                    object_pairs_hook=reject_duplicate_keys,
                )
            except (UnicodeDecodeError, json.JSONDecodeError, DuplicateJsonKey) as error:
                messages.put(
                    (
                        "error",
                        NativeSensorMalformed(
                            "Codex app-server returned malformed or ambiguous JSONL"
                        ),
                    )
                )
                return
            messages.put(("value", value))
    except OSError:
        messages.put(("error", GuardError("Codex app-server output read failed")))


def _stderr_reader(stream: Any, messages: queue.Queue[tuple[str, Any]]) -> None:
    retained = 0
    try:
        while True:
            chunk = stream.read(8192)
            if not chunk:
                return
            retained += len(chunk)
            if retained > MAX_OUTPUT_BYTES:
                messages.put(
                    (
                        "error",
                        GuardError("Codex app-server stderr exceeded the source guard limit"),
                    )
                )
                return
    except OSError:
        messages.put(("error", GuardError("Codex app-server stderr read failed")))


def _send_jsonl(process: subprocess.Popen[bytes], value: dict[str, Any]) -> None:
    if process.stdin is None:
        raise GuardError("Codex app-server stdin is unavailable")
    payload = (
        json.dumps(value, separators=(",", ":"), ensure_ascii=True) + "\n"
    ).encode("utf-8")
    try:
        process.stdin.write(payload)
        process.stdin.flush()
    except (BrokenPipeError, OSError) as error:
        raise GuardError("Codex app-server request write failed") from error


def _await_response(
    process: subprocess.Popen[bytes],
    messages: queue.Queue[tuple[str, Any]],
    request_id: int,
    deadline: float,
) -> dict[str, Any]:
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise GuardError("Codex app-server request timed out")
        try:
            kind, payload = messages.get(timeout=remaining)
        except queue.Empty as error:
            raise GuardError("Codex app-server request timed out") from error
        if kind == "error":
            raise payload
        if kind == "eof":
            raise GuardError(
                f"Codex app-server exited before response {request_id} "
                f"(status {process.poll()})"
            )
        if not isinstance(payload, dict):
            raise NativeSensorMalformed(
                "Codex app-server emitted a non-object JSONL message"
            )
        response_id = payload.get("id")
        if response_id is None:
            continue
        if isinstance(response_id, bool) or response_id != request_id:
            raise NativeSensorMalformed(
                "Codex app-server returned an unexpected response id"
            )
        if "error" in payload:
            raise NativeSensorMalformed(
                "Codex app-server returned a protocol error"
            )
        result = payload.get("result")
        if not isinstance(result, dict):
            raise NativeSensorMalformed(
                "Codex app-server response omitted an object result"
            )
        return result


def read_codex_native_payload() -> tuple[dict[str, Any], dict[str, Any], str]:
    executable, identity, version = qualify_codex()
    verify_native_identity(executable, identity, "before app-server launch")
    try:
        process = subprocess.Popen(
            [str(executable), "app-server", "--stdio"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            close_fds=True,
        )
    except OSError as error:
        if isinstance(error, FileNotFoundError):
            raise NativeSensorUnavailable(
                "Codex app-server is unavailable"
            ) from error
        raise NativeSensorError("Codex app-server launch failed") from error
    if process.stdout is None or process.stderr is None:
        process.kill()
        process.wait()
        raise GuardError("Codex app-server pipes are unavailable")
    messages: queue.Queue[tuple[str, Any]] = queue.Queue()
    stdout_thread = threading.Thread(
        target=_jsonl_reader,
        args=(process.stdout, messages),
        daemon=True,
    )
    stderr_thread = threading.Thread(
        target=_stderr_reader,
        args=(process.stderr, messages),
        daemon=True,
    )
    stdout_thread.start()
    stderr_thread.start()
    deadline = time.monotonic() + CODEX_APP_SERVER_TIMEOUT_SECONDS
    error: BaseException | None = None
    result: tuple[dict[str, Any], dict[str, Any], str] | None = None
    try:
        _send_jsonl(
            process,
            {
                "method": "initialize",
                "id": 1,
                "params": {
                    "clientInfo": {
                        "name": "aigent-hive",
                        "title": "Aigent Hive",
                        "version": "0.7.0",
                    }
                },
            },
        )
        _await_response(process, messages, 1, deadline)
        _send_jsonl(process, {"method": "initialized", "params": {}})
        _send_jsonl(
            process,
            {
                "method": "account/read",
                "id": 2,
                "params": {"refreshToken": False},
            },
        )
        account = _await_response(process, messages, 2, deadline)
        _send_jsonl(
            process,
            {
                "method": "account/rateLimits/read",
                "id": 3,
                "params": None,
            },
        )
        rate_limits = _await_response(process, messages, 3, deadline)
        result = (account, rate_limits, version)
    except BaseException as caught:
        error = caught
    finally:
        if process.stdin is not None:
            try:
                process.stdin.close()
            except OSError:
                pass
        try:
            process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            process.terminate()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        stdout_thread.join(timeout=1)
        stderr_thread.join(timeout=1)
    verify_native_identity(executable, identity, "during app-server exchange")
    if error is not None:
        if isinstance(error, GuardError):
            raise error
        raise GuardError("Codex app-server exchange failed") from error
    assert result is not None
    return result


def _codex_window(
    value: Any,
    *,
    measured_at: dt.datetime,
) -> tuple[str, float, int]:
    if not isinstance(value, dict):
        raise NativeSensorMalformed(
            "Codex native sensor returned a malformed quota window"
        )
    used = value.get("usedPercent")
    if (
        isinstance(used, bool)
        or not isinstance(used, (int, float))
        or not math.isfinite(float(used))
        or not 0 <= float(used) <= 100
    ):
        raise NativeSensorMalformed(
            "Codex native sensor returned an invalid usedPercent"
        )
    minutes = value.get("windowDurationMins")
    if isinstance(minutes, bool) or not isinstance(minutes, int):
        raise NativeSensorMalformed(
            "Codex native sensor returned an invalid quota duration"
        )
    if minutes == 300:
        name = "session"
    elif minutes == 10080:
        name = "weekly"
    else:
        raise NativeSensorMalformed(
            "Codex native sensor returned an unexpected quota duration"
        )
    resets_at = value.get("resetsAt")
    if (
        isinstance(resets_at, bool)
        or not isinstance(resets_at, int)
        or resets_at <= int(measured_at.timestamp())
    ):
        raise NativeSensorMalformed(
            "Codex native sensor returned a stale quota window"
        )
    return name, float(used), resets_at


def read_codex_native_quota() -> dict[str, Any]:
    account_result, response, version = read_codex_native_payload()
    account = account_result.get("account")
    if not isinstance(account, dict) or account.get("type") != "chatgpt":
        raise NativeSensorUnavailable(
            "Codex native sensor has no subscription account"
        )
    email = account.get("email")
    account_plan = account.get("planType")
    if not isinstance(email, str) or not email.strip():
        raise NativeSensorMalformed(
            "Codex native sensor omitted account identity"
        )
    if account_plan not in SUPPORTED_CODEX_PLANS:
        raise NativeSensorUnsupported(
            "Codex native sensor returned an unsupported account plan"
        )
    rate_limits_by_id = response.get("rateLimitsByLimitId")
    selected = None
    if isinstance(rate_limits_by_id, dict):
        selected = rate_limits_by_id.get("codex")
    legacy = response.get("rateLimits")
    if selected is None:
        selected = legacy
    elif legacy is not None and legacy != selected:
        raise NativeSensorMalformed(
            "Codex native sensor returned conflicting quota payloads"
        )
    if not isinstance(selected, dict) or selected.get("limitId") != "codex":
        raise NativeSensorMalformed(
            "Codex native sensor returned an unexpected limitId"
        )
    if selected.get("planType") != account_plan:
        raise NativeSensorMalformed(
            "Codex native sensor returned a conflicting quota plan"
        )
    measured_at = utc_now()
    windows: dict[str, tuple[float, int]] = {}
    for field in ("primary", "secondary"):
        raw = selected.get(field)
        if raw is None:
            continue
        name, used, resets_at = _codex_window(raw, measured_at=measured_at)
        if name in windows:
            raise NativeSensorMalformed(
                "Codex native sensor returned duplicate quota windows"
            )
        windows[name] = (used, resets_at)
    if "session" in windows:
        window_name = "session"
    elif "weekly" in windows:
        window_name = "weekly"
    else:
        raise NativeSensorMalformed(
            "Codex native sensor omitted supported quota windows"
        )
    used, resets_at = windows[window_name]
    return {
        "sensor": CODEX_NATIVE_SENSOR,
        "sensor_version": version,
        "window": window_name,
        "used_percent": used,
        "remaining_percent": round(100.0 - used, 6),
        "measured_at": measured_at.isoformat().replace("+00:00", "Z"),
        "resets_at": resets_at,
        "account_digest": "sha256:"
        + hashlib.sha256(email.strip().encode("utf-8")).hexdigest(),
        "fallback_used": False,
    }


def read_codexbar_quota() -> dict[str, Any]:
    executable, identity = qualify_codexbar()
    if executable_identity(executable) != identity:
        raise GuardError("CodexBar executable changed before usage read")
    payload = run_bounded(
        executable,
        USAGE_ARGUMENTS,
        60,
        sensor_name="CodexBar",
    )
    if executable_identity(executable) != identity:
        raise GuardError("CodexBar executable changed during usage read")
    try:
        rows = json.loads(payload, object_pairs_hook=reject_duplicate_keys)
    except (json.JSONDecodeError, DuplicateJsonKey) as error:
        raise GuardError("CodexBar returned malformed or ambiguous usage JSON") from error
    if not isinstance(rows, list) or len(rows) != 1 or not isinstance(rows[0], dict):
        raise GuardError("CodexBar must return exactly one active Codex account")
    row = rows[0]
    if row.get("provider") != "codex" or row.get("source") != "codex-cli":
        raise GuardError("CodexBar returned a non-local or wrong-provider row")
    usage = row.get("usage")
    if not isinstance(usage, dict):
        raise GuardError("CodexBar omitted usage")
    measured_at = parse_timestamp(usage.get("updatedAt"))
    age = (utc_now() - measured_at).total_seconds()
    if age < -30 or age > MAX_SNAPSHOT_AGE_SECONDS:
        raise GuardError("CodexBar usage snapshot is stale")
    if "primary" not in usage:
        raise GuardError("CodexBar omitted the primary session window")
    primary = usage.get("primary")
    if primary is None:
        window_name = "weekly"
        expected_minutes = 10080
        window = usage.get("secondary")
    else:
        window_name = "session"
        expected_minutes = 300
        window = primary
    if not isinstance(window, dict):
        raise GuardError(f"CodexBar omitted the {window_name} usage window")
    used = window.get("usedPercent")
    if isinstance(used, bool) or not isinstance(used, (int, float)) or not 0 <= used <= 100:
        raise GuardError("CodexBar returned an invalid usedPercent")
    window_minutes = window.get("windowMinutes")
    if (
        isinstance(window_minutes, bool)
        or not isinstance(window_minutes, int)
        or window_minutes != expected_minutes
    ):
        raise GuardError(f"CodexBar returned the wrong {window_name} quota window")
    remaining = round(100.0 - float(used), 6)
    return {
        "sensor": "codexbar",
        "sensor_version": CODEXBAR_VERSION,
        "window": window_name,
        "used_percent": float(used),
        "remaining_percent": remaining,
        "measured_at": measured_at.isoformat().replace("+00:00", "Z"),
        "fallback_used": True,
    }


def read_quota() -> dict[str, Any]:
    try:
        return read_codex_native_quota()
    except (
        NativeSensorUnavailable,
        NativeSensorUnsupported,
        NativeSensorMalformed,
    ) as native_error:
        try:
            quota = read_codexbar_quota()
        except CodexBarUnavailable as fallback_error:
            raise CodexBarUnavailable(
                "Codex native sensor is unavailable and the optional "
                "CodexBar fallback is not installed"
            ) from fallback_error
        except GuardError as fallback_error:
            raise GuardError(
                "Codex native sensor is unavailable and the optional CodexBar "
                f"fallback failed ({fallback_error})"
            ) from native_error
        quota["native_sensor"] = "unavailable"
        quota["fallback_reason"] = "native-unavailable"
        return quota


def unknown_retry_delay_seconds() -> float:
    """Return the production retry delay, with a fixture-only test override."""

    if "HIVE_SOURCE_USAGE_FIXTURE" not in os.environ:
        return UNKNOWN_RETRY_DELAY_SECONDS
    raw = os.environ.get("HIVE_USAGE_UNKNOWN_RETRY_DELAY_SECONDS")
    if raw is None:
        return UNKNOWN_RETRY_DELAY_SECONDS
    try:
        value = float(raw)
    except ValueError:
        return UNKNOWN_RETRY_DELAY_SECONDS
    if not math.isfinite(value) or not 0 <= value <= UNKNOWN_RETRY_DELAY_SECONDS:
        return UNKNOWN_RETRY_DELAY_SECONDS
    return value


def apply_sensor_unknown(
    decision: dict[str, Any],
    error: GuardError,
) -> None:
    decision["quota_decision"] = "usage_unknown"
    decision["reason"] = str(error)
    if not isinstance(error, CodexBarUnavailable):
        return
    decision["notification"] = (
        "CodexBar fallback is not installed for codex; installation is "
        "optional and requires explicit current-action consent."
    )
    decision["fallback_install"] = {
        "provider": "codex",
        "fallback": "codexbar",
        "availability": "missing",
        "command_preview": source_guard_command("codex", apply=False),
        "decline_effect": "core-usable-automatic-dispatch-usage-unknown",
    }
    decision["next_action"] = source_guard_command("codex", apply=False)


def evaluate(root: Path, expected_session_id: str | None = None) -> tuple[dict[str, Any], int]:
    session = load_current_session(root, expected_session_id)
    settings = load_settings(root)
    enabled = guard_enabled(root, session)
    decision: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "session_id": session["session_id"],
        "guard_enabled": enabled,
        "threshold_remaining_percent": settings["threshold_remaining_percent"],
        "checked_at": iso_now(),
    }
    quota_exit = EXIT_ALLOWED
    quota: dict[str, Any] | None = None
    sensor_error: GuardError | None = None
    retry_count = 0
    retry_delay = unknown_retry_delay_seconds()
    transient_unknown = False
    for attempt in range(UNKNOWN_RETRY_COUNT + 1):
        try:
            quota = read_quota()
            break
        except NativeSensorIntegrity as error:
            sensor_error = error
            break
        except GuardError as error:
            sensor_error = error
            if attempt == UNKNOWN_RETRY_COUNT:
                transient_unknown = True
                break
            retry_count += 1
            time.sleep(retry_delay)

    if quota is not None:
        decision.update(quota)
        if quota["remaining_percent"] <= settings["threshold_remaining_percent"]:
            decision["quota_decision"] = "halted"
            quota_exit = EXIT_HALTED
        else:
            decision["quota_decision"] = "allowed"
        if retry_count:
            decision["transient_unknown_recovered"] = True
    else:
        if sensor_error is None:
            raise GuardError("usage sensor failed without an error")
        apply_sensor_unknown(decision, sensor_error)
        if transient_unknown:
            decision["transient_unknown_ignored"] = True
        else:
            quota_exit = EXIT_UNKNOWN
    if retry_count:
        decision["unknown_retry_count"] = retry_count
        decision["unknown_retry_delay_seconds"] = retry_delay

    halt = halt_path(root, session["session_id"])
    preserve_confirmed_halt = False
    if enabled and quota is None:
        existing_halt = read_json(halt)
        preserve_confirmed_halt = (
            existing_halt is not None
            and existing_halt.get("schema_version") == SCHEMA_VERSION
            and existing_halt.get("session_id") == session["session_id"]
            and existing_halt.get("decision") == "halted"
        )
    if enabled:
        if preserve_confirmed_halt:
            decision["confirmed_halt_preserved"] = True
            if transient_unknown:
                decision["enforcement_decision"] = "halted"
                exit_code = EXIT_HALTED
            else:
                decision["enforcement_decision"] = decision["quota_decision"]
                exit_code = quota_exit
        elif transient_unknown:
            decision["enforcement_decision"] = "allowed"
            exit_code = EXIT_ALLOWED
        else:
            decision["enforcement_decision"] = decision["quota_decision"]
            exit_code = quota_exit
    else:
        decision["enforcement_decision"] = "session_bypass"
        exit_code = EXIT_ALLOWED
    write_json(root, observation_path(root, session["session_id"]), decision)
    if preserve_confirmed_halt:
        pass
    elif enabled and quota_exit in (EXIT_HALTED, EXIT_UNKNOWN):
        write_json(
            root,
            halt,
            {
                "schema_version": SCHEMA_VERSION,
                "session_id": session["session_id"],
                "decision": decision["quota_decision"],
                "threshold_remaining_percent": settings["threshold_remaining_percent"],
                "created_at": decision["checked_at"],
            },
        )
    else:
        remove_owned_file(root, halt)
    return decision, exit_code


def command_matches_watcher(pid: int, script: Path, session_id: str) -> bool:
    if not process_is_alive(pid):
        return False
    try:
        completed = subprocess.run(
            ["ps", "-p", str(pid), "-o", "command="],
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=5,
            text=True,
        )
    except (OSError, subprocess.TimeoutExpired):
        return False
    command = completed.stdout.strip()
    return (
        completed.returncode == 0
        and str(script.resolve()) in command
        and " watch " in f" {command} "
        and session_id in command
    )


def watcher_status(root: Path, session: dict[str, Any]) -> dict[str, Any]:
    value = read_json(watcher_path(root, session["session_id"]))
    if value is None:
        return {"active": False}
    pid = value.get("pid")
    if isinstance(pid, bool) or not isinstance(pid, int):
        raise GuardError("invalid watcher state")
    active = command_matches_watcher(pid, Path(__file__), session["session_id"])
    return {
        "active": active,
        "pid": pid if active else None,
        "started_at": value.get("started_at"),
    }


def start_watcher(root: Path, session: dict[str, Any]) -> dict[str, Any]:
    status = watcher_status(root, session)
    if status["active"]:
        return status
    log_path = watcher_log_path(root, session["session_id"])
    assert_safe_parent(root, log_path)
    if log_path.exists() and log_path.is_symlink():
        raise GuardError("refusing symlink watcher log")
    flags = os.O_WRONLY | os.O_CREAT | os.O_APPEND
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        log_descriptor = os.open(log_path, flags, 0o600)
    except OSError as error:
        raise GuardError("cannot open watcher log safely") from error
    if not stat.S_ISREG(os.fstat(log_descriptor).st_mode):
        os.close(log_descriptor)
        raise GuardError("watcher log is not a regular file")
    log = os.fdopen(log_descriptor, "ab", buffering=0)
    arguments = [
        sys.executable,
        str(Path(__file__).resolve()),
        "--root",
        str(root),
        "--session-id",
        session["session_id"],
        "watch",
    ]
    try:
        process = subprocess.Popen(
            arguments,
            stdin=subprocess.DEVNULL,
            stdout=log,
            stderr=log,
            cwd=root,
            start_new_session=True,
            close_fds=True,
        )
    finally:
        log.close()
    value = {
        "schema_version": SCHEMA_VERSION,
        "session_id": session["session_id"],
        "session_pid": session["pid"],
        "pid": process.pid,
        "started_at": iso_now(),
        "script": str(Path(__file__).resolve()),
    }
    write_json(root, watcher_path(root, session["session_id"]), value)
    time.sleep(0.1)
    status = watcher_status(root, session)
    if not status["active"]:
        raise GuardError("usage-guard watcher failed to start")
    return status


def stop_watcher(root: Path, session: dict[str, Any]) -> dict[str, Any]:
    status = watcher_status(root, session)
    if not status["active"]:
        remove_owned_file(root, watcher_path(root, session["session_id"]))
        return {"active": False, "stopped": False}
    pid = int(status["pid"])
    os.kill(pid, signal.SIGTERM)
    for _ in range(50):
        if not command_matches_watcher(pid, Path(__file__), session["session_id"]):
            break
        time.sleep(0.02)
    if command_matches_watcher(pid, Path(__file__), session["session_id"]):
        raise GuardError("verified watcher did not stop")
    remove_owned_file(root, watcher_path(root, session["session_id"]))
    return {"active": False, "stopped": True}


def watch(root: Path, session_id: str, once: bool) -> int:
    while True:
        try:
            decision, exit_code = evaluate(root, session_id)
            print(json.dumps(decision, sort_keys=True), flush=True)
        except SessionExpired as error:
            print(json.dumps({"watcher": "expired", "reason": str(error)}), flush=True)
            return EXIT_ALLOWED
        except GuardError as error:
            print(json.dumps({"watcher": "error", "reason": str(error)}), flush=True)
            exit_code = EXIT_UNKNOWN
        if once:
            return exit_code
        try:
            poll_seconds = load_settings(root)["poll_seconds"]
        except GuardError:
            poll_seconds = DEFAULT_POLL_SECONDS
        time.sleep(poll_seconds)


def emit(value: dict[str, Any], json_output: bool) -> None:
    if json_output:
        print(json.dumps(value, sort_keys=True))
        return
    for key, item in value.items():
        print(f"{key}: {item}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", help=argparse.SUPPRESS)
    parser.add_argument("--session-id", help=argparse.SUPPRESS)
    parser.add_argument("--json", action="store_true")
    commands = parser.add_subparsers(dest="command", required=True)
    def command(name: str, **kwargs: Any) -> argparse.ArgumentParser:
        child = commands.add_parser(name, **kwargs)
        child.add_argument("--json", action="store_true", default=argparse.SUPPRESS)
        return child

    command("check")
    command("gate")
    command("status")
    threshold = command("set-threshold")
    threshold.add_argument("percent", type=int)
    disable = command("session-disable")
    disable.add_argument("--confirm-session-disable", action="store_true")
    command("session-enable")
    toggle = command("session-toggle")
    toggle.add_argument("--confirm-session-disable", action="store_true")
    command("watch-start")
    command("watch-status")
    command("watch-stop")
    install = command("fallback-install")
    install.add_argument("--host", choices=FALLBACK_HOSTS, required=True)
    mode = install.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dry-run", action="store_true")
    mode.add_argument("--apply", action="store_true")
    install.add_argument("--confirm-install", action="store_true")
    watcher = command("watch", help=argparse.SUPPRESS)
    watcher.add_argument("--once", action="store_true", help=argparse.SUPPRESS)
    return parser


def main() -> int:
    arguments = build_parser().parse_args()
    try:
        root = discover_root(arguments.root)
        if arguments.command == "watch":
            session_id = arguments.session_id
            if session_id is None:
                session_id = load_current_session(root)["session_id"]
            return watch(root, session_id, arguments.once)
        if arguments.command == "fallback-install":
            result = fallback_install(
                arguments.host,
                apply=arguments.apply,
                confirmed=arguments.confirm_install,
            )
            emit(result, arguments.json)
            return EXIT_ALLOWED
        session = load_current_session(root, arguments.session_id)
        if arguments.command == "gate":
            watcher = start_watcher(root, session)
            decision, exit_code = evaluate(root, session["session_id"])
            decision["watcher"] = watcher
            decision["halt_marker"] = halt_path(root, session["session_id"]).exists()
            emit(decision, arguments.json)
            return exit_code
        if arguments.command in ("check", "status"):
            decision, exit_code = evaluate(root, session["session_id"])
            decision["watcher"] = watcher_status(root, session)
            decision["halt_marker"] = halt_path(root, session["session_id"]).exists()
            emit(decision, arguments.json)
            return exit_code
        if arguments.command == "set-threshold":
            result = save_threshold(root, arguments.percent)
        elif arguments.command == "session-disable":
            result = set_session_enabled(
                root,
                session,
                False,
                confirmed=arguments.confirm_session_disable,
            )
        elif arguments.command == "session-enable":
            result = set_session_enabled(root, session, True, confirmed=True)
        elif arguments.command == "session-toggle":
            result = set_session_enabled(
                root,
                session,
                not guard_enabled(root, session),
                confirmed=arguments.confirm_session_disable,
            )
        elif arguments.command == "watch-start":
            result = start_watcher(root, session)
        elif arguments.command == "watch-status":
            result = watcher_status(root, session)
        elif arguments.command == "watch-stop":
            result = stop_watcher(root, session)
        else:
            raise GuardError(f"unsupported command: {arguments.command}")
        emit(result, arguments.json)
        return EXIT_ALLOWED
    except SessionExpired as error:
        emit({"status": "session_expired", "reason": str(error)}, arguments.json)
        return EXIT_UNKNOWN
    except FallbackInstallUnsupported as error:
        emit(
            {
                "schema_version": SCHEMA_VERSION,
                "action": "InstallUsageFallback",
                "status": "unsupported",
                "code": "hive.usage-fallback-install-unsupported",
                "reason": str(error),
            },
            arguments.json,
        )
        return EXIT_USAGE
    except FallbackInstallInvalid as error:
        emit(
            {
                "schema_version": SCHEMA_VERSION,
                "action": "InstallUsageFallback",
                "status": "error",
                "code": "hive.invalid-input",
                "reason": str(error),
            },
            arguments.json,
        )
        return EXIT_USAGE
    except GuardError as error:
        emit({"status": "usage_unknown", "reason": str(error)}, arguments.json)
        if arguments.command in {
            "set-threshold",
            "session-disable",
            "session-enable",
            "session-toggle",
            "watch-start",
            "watch-stop",
            "fallback-install",
        }:
            return EXIT_USAGE
        return EXIT_UNKNOWN


if __name__ == "__main__":
    raise SystemExit(main())
