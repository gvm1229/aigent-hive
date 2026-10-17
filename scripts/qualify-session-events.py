"""Read only owned acceptance subjects' native turn markers; never copies conversation text."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/work/usage-monitor-acceptance"
IDENTITY = re.compile(r"[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}")


def digest(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode()).hexdigest()


def native_path(value) -> Path:
    text = str(value)
    if text.startswith("\\\\?\\UNC\\"):
        text = "\\\\" + text[8:]
    elif text.startswith("\\\\?\\"):
        text = text[4:]
    return Path(text).resolve(strict=True)


def scan_turn(path: Path) -> dict:
    turn = None
    status = "missing"
    tool_calls = 0
    marker_at = None
    pending = set()
    with path.open("rb") as stream:
        while line := stream.readline(1048576):
            if not line.endswith(b"\n"):
                while line and not line.endswith(b"\n"):
                    line = stream.readline(1048576)
                continue
            if b'"event_msg"' not in line and b'function_call' not in line and b'custom_tool_call' not in line:
                continue
            try:
                item = json.loads(line)
            except (ValueError, UnicodeError):
                continue  # A partially written final marker never confirms a stop.
            payload = item.get("payload") or {}
            kind = payload.get("type")
            if item.get("type") == "event_msg":
                if kind == "task_started" and payload.get("turn_id"):
                    turn = payload["turn_id"]
                    status = "running"
                    tool_calls = 0
                    marker_at = item.get("timestamp")
                    pending.clear()
                elif payload.get("turn_id") == turn:
                    if kind == "turn_aborted":
                        status = "interrupted"
                        marker_at = item.get("timestamp")
                    elif kind == "task_complete":
                        status = "completed"
                        marker_at = item.get("timestamp")
            elif item.get("type") == "response_item" and kind in ("function_call", "custom_tool_call") and turn:
                tool_calls += 1
                pending.add(payload.get("call_id"))
            elif item.get("type") == "response_item" and kind in ("function_call_output", "custom_tool_call_output"):
                pending.discard(payload.get("call_id"))
    return {"turn_id": turn, "status": status, "tool_calls": tool_calls,
            "pending_tools": len(pending), "marker_at": marker_at}


def subject_state(thread: str) -> dict:
    if not IDENTITY.fullmatch(thread):
        raise ValueError("Invalid acceptance thread identity")
    home = native_path(os.environ.get("CODEX_HOME") or Path.home() / ".codex")
    database = home / "state_5.sqlite"
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, timeout=2) as connection:
        row = connection.execute("SELECT rollout_path FROM threads WHERE id = ?", (thread,)).fetchone()
    if not row:
        raise ValueError("Acceptance subject is absent from native metadata")
    path = native_path(row[0])
    if not path.is_relative_to(home) or thread not in path.name:
        raise ValueError("Native subject path escaped its exact ownership")
    with path.open("rb") as stream:
        metadata = json.loads(stream.readline(1048576))
    if metadata.get("type") != "session_meta" or (metadata.get("payload") or {}).get("id") != thread:
        raise ValueError("Native session metadata differs from the acceptance subject")
    return scan_turn(path)


def evaluate(subject: dict, current: dict, capture: bool) -> dict:
    same_turn = capture or current["turn_id"] == subject["turn_id"]
    tool_free = not subject.get("tool_free") or (
        current.get("pending_tools", 0) == 0 if capture
        else current["tool_calls"] == subject.get("tool_calls", 0)
    )
    expected = "running" if capture else "interrupted"
    return {
        "role": subject.get("role", "unspecified"),
        "tool_free_required": bool(subject.get("tool_free")),
        "thread_digest": digest(subject["thread"]),
        "turn_digest": digest(current["turn_id"] or "missing"),
        "status": current["status"], "same_turn": same_turn,
        "native_marker_at": current.get("marker_at"),
        "tool_free_after_capture": tool_free,
        "verified": current["status"] == expected and same_turn and tool_free,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("capture", "verify"))
    parser.add_argument("--subjects", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source = args.subjects.resolve()
    output = args.output.resolve()
    if not source.is_relative_to(FIXTURES) or not output.is_relative_to(FIXTURES):
        parser.error("Acceptance metadata outside owned fixtures")
    subjects = json.loads(source.read_text(encoding="utf-8-sig"))["subjects"]
    if not 2 <= len(subjects) <= 8 or len({s["thread"] for s in subjects}) != len(subjects):
        parser.error("Expected two to eight distinct acceptance subjects")
    roles={s.get("role") for s in subjects}
    if not {"parent","reasoning","shell"}.issubset(roles) or not roles.issubset({"parent","reasoning","shell","subagent"}):
        parser.error("Exact parent, tool-free reasoning and shell roles required")
    if any(s.get("role")=="reasoning" and s.get("tool_free") is not True for s in subjects):
        parser.error("Reasoning subject must prove tool-free execution")
    results = []
    bindings = []
    capture = args.mode == "capture"
    for subject in subjects:
        current = subject_state(subject["thread"])
        results.append(evaluate(subject, current, capture))
        bindings.append({**subject, "turn_id": current["turn_id"], "tool_calls": current["tool_calls"]})
    record = {"schema_version": 1, "mode": args.mode,
              "verified": all(row["verified"] for row in results), "subjects": results}
    # Raw IDs stay only in this disposable private binding, never public receipts.
    stored = {**record, "subjects": bindings} if capture else record
    output.write_text(json.dumps(stored, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(record, separators=(",", ":")))
    return 0 if record["verified"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
