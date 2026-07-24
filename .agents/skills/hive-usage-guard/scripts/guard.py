#!/usr/bin/env python3
"""Session-bound source-development usage guard backed by local CodexBar data."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
CODEXBAR_VERSION = "0.45.2"
DEFAULT_THRESHOLD = 10
DEFAULT_POLL_SECONDS = 15
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


class DuplicateJsonKey(ValueError):
    """CodexBar JSON contained an ambiguous duplicate object key."""


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


def run_bounded(executable: Path, arguments: tuple[str, ...], timeout: int) -> str:
    try:
        completed = subprocess.run(
            [str(executable), *arguments],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=timeout,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GuardError("CodexBar command failed or timed out") from error
    if len(completed.stdout) > MAX_OUTPUT_BYTES or len(completed.stderr) > MAX_OUTPUT_BYTES:
        raise GuardError("CodexBar output exceeded the source guard limit")
    if completed.returncode != 0:
        raise GuardError("CodexBar returned a non-zero status")
    try:
        return completed.stdout.decode("utf-8")
    except UnicodeDecodeError as error:
        raise GuardError("CodexBar output was not UTF-8") from error


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
        raise GuardError("CodexBar is unavailable")
    executable = Path(candidate).resolve()
    if not executable.is_file() or not os.access(executable, os.X_OK):
        raise GuardError("CodexBar executable is invalid")
    identity = executable_identity(executable)
    version = run_bounded(executable, ("--version",), 5).strip()
    if executable_identity(executable) != identity:
        raise GuardError("CodexBar executable changed during qualification")
    if version != f"CodexBar {CODEXBAR_VERSION}":
        raise GuardError("CodexBar version is unsupported")
    return executable, identity


def read_quota() -> dict[str, Any]:
    executable, identity = qualify_codexbar()
    if executable_identity(executable) != identity:
        raise GuardError("CodexBar executable changed before usage read")
    payload = run_bounded(executable, USAGE_ARGUMENTS, 60)
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
    }


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
    try:
        quota = read_quota()
        decision.update(quota)
        if quota["remaining_percent"] <= settings["threshold_remaining_percent"]:
            decision["quota_decision"] = "halted"
            quota_exit = EXIT_HALTED
        else:
            decision["quota_decision"] = "allowed"
    except GuardError as error:
        decision["quota_decision"] = "usage_unknown"
        decision["reason"] = str(error)
        quota_exit = EXIT_UNKNOWN
    if enabled:
        decision["enforcement_decision"] = decision["quota_decision"]
        exit_code = quota_exit
    else:
        decision["enforcement_decision"] = "session_bypass"
        exit_code = EXIT_ALLOWED
    write_json(root, observation_path(root, session["session_id"]), decision)
    halt = halt_path(root, session["session_id"])
    if enabled and quota_exit in (EXIT_HALTED, EXIT_UNKNOWN):
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
    except GuardError as error:
        emit({"status": "usage_unknown", "reason": str(error)}, arguments.json)
        if arguments.command in {
            "set-threshold",
            "session-disable",
            "session-enable",
            "session-toggle",
            "watch-start",
            "watch-stop",
        }:
            return EXIT_USAGE
        return EXIT_UNKNOWN


if __name__ == "__main__":
    raise SystemExit(main())
