#!/usr/bin/env python3
"""Measure frozen native hook wire cases; never report live host enforcement."""
import argparse
import hashlib
import json
import platform
import statistics
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/policy/holdout-0.11.0/cases.json"


def digest(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


def snapshot(root):
    return {p.relative_to(root).as_posix(): digest(p.read_bytes()) if p.is_file() else "directory"
            for p in root.rglob("*")}


def payload(host, case, target):
    operation = case["operation"]
    if operation in ("stop", "malformed"):
        return case["raw"]
    if operation == "shell":
        if host == "antigravity":
            return json.dumps({"workspacePaths": [str(target)], "toolCall": {
                "name": "run_command", "args": {"CommandLine": case["command"]}}})
        return json.dumps({"cwd": str(target), "hook_event_name": "PreToolUse",
                           "tool_name": "exec_command" if host == "codex" else "Bash",
                           "tool_input": {"command": case["command"]}})
    path = case["path"]
    if host == "antigravity":
        value = {"workspacePaths": [str(target)], "toolCall": {
            "name": "write_to_file", "args": {"TargetFile": path, "CodeContent": "synthetic"}}}
    elif host == "claude":
        value = {"cwd": str(target), "hook_event_name": "PreToolUse", "tool_name": "Write",
                 "tool_input": {"file_path": path, "content": "synthetic"}}
    else:
        value = {"cwd": str(target), "hook_event_name": "PreToolUse", "tool_name": "apply_patch",
                 "tool_input": {"command": f"*** Begin Patch\n*** Delete File: {path}\n*** End Patch"}}
    return json.dumps(value)


def classify(host, value, event):
    if host == "antigravity":
        expected = "allow" if event == "Stop" else "ask"
        if value.get("decision") == "deny":
            return "deny"
        if value.get("decision") == expected and "permissionOverrides" not in value:
            return "neutral-stop" if event == "Stop" else "permission-neutral"
    elif value == {}:
        return "neutral-stop" if event == "Stop" else "permission-neutral"
    elif value.get("hookSpecificOutput", {}).get("permissionDecision") == "deny":
        return "deny"
    return "unexpected"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    work = args.work.resolve()
    if not work.is_relative_to((ROOT / "tests/work").resolve()) or work.exists():
        raise ValueError("choose a new directory beneath tests/work")
    binary = args.binary.resolve()
    frozen = subprocess.check_output(["git", "show", "7c99a496:tests/fixtures/policy/holdout-0.11.0/cases.json"], cwd=ROOT)
    if FIXTURE.read_bytes() != frozen:
        raise ValueError("frozen holdout changed; review and freeze a new set")
    data = json.loads(frozen)
    target = work / "consumer"
    (target / ".hive/config").mkdir(parents=True)
    (target / ".hive/config/harness.toml").write_bytes(b"synthetic sentinel\n")
    before = snapshot(target)
    rows = []
    for host in data["hosts"]:
        for case in data["cases"]:
            event = "Stop" if case["operation"] == "stop" else "PreToolUse"
            raw = payload(host, case, target)
            timings, outputs = [], []
            for _ in range(5):
                started = time.perf_counter()
                result = subprocess.run([str(binary), "policy", "hook", "--host", host,
                    "--event", event, "--target", str(target), "--stdin-json"], input=raw,
                    capture_output=True, text=True, encoding="utf-8", timeout=15)
                timings.append((time.perf_counter() - started) * 1000)
                try:
                    actual = classify(host, json.loads(result.stdout), event)
                except (ValueError, AttributeError):
                    actual = "invalid-response"
                safe = data["rules"]["forbidden_output"] not in result.stdout + result.stderr
                outputs.append((result.returncode, actual, safe))
            passed = all(code == 0 and actual == case["expect"] and safe for code, actual, safe in outputs)
            rows.append({"host": host, "case": case["id"], "expected": case["expect"],
                         "observed": outputs, "passed": passed, "samples_ms": timings,
                         "median_ms": statistics.median(timings), "input_bytes": len(raw.encode()),
                         "output_bytes": len(result.stdout.encode()), "calls": len(timings)})
    unchanged = snapshot(target) == before
    report = {"schema_version": 1, "fixture_digest": digest(frozen),
        "binary_digest": digest(binary.read_bytes()), "platform": platform.platform(),
        "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "rows": rows, "target_unchanged": unchanged,
        "passed": sum(row["passed"] for row in rows), "total": len(rows),
        "live_host_status": "unverified", "instruction_only_baseline": "not-run",
        "limit": "CLI processes only; host load, trust, tool execution, timeout handling, cancellation and model compliance not measured"}
    (work / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2)+"\n", encoding="utf-8")
    print(json.dumps({"passed": report["passed"], "total": len(rows), "target_unchanged": unchanged}))
    return int(not unchanged or report["passed"] != len(rows))


if __name__ == "__main__":
    raise SystemExit(main())
