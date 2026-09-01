#!/usr/bin/env python3
"""Run one disposable verified-workflow acceptance and emit one JSON receipt."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
ROUTE_FIXTURE = ROOT / "tests/fixtures/skills/routes/complex-verified-workflow.json"
JUDGE_FIXTURES = ROOT / "tests/fixtures/judge"


class AcceptanceFailure(RuntimeError):
    """One required acceptance observation was absent."""


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def markdown_document(frontmatter: dict[str, Any], body: str) -> bytes:
    return b"---\n" + canonical_bytes(frontmatter) + b"\n---\n" + body.encode("utf-8")


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_bytes(value) + b"\n")


def invoke(
    command: list[str],
    *,
    cwd: Path,
    expect_json: bool = False,
) -> dict[str, Any]:
    process = subprocess.Popen(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    stdout, stderr = process.communicate(timeout=120)
    result: dict[str, Any] = {
        "command": command,
        "process_id": process.pid,
        "return_code": process.returncode,
        "stdout_digest": digest_bytes(stdout.encode("utf-8")),
        "stderr_digest": digest_bytes(stderr.encode("utf-8")),
    }
    if expect_json:
        try:
            payload = json.loads(stdout)
        except json.JSONDecodeError as error:
            raise AcceptanceFailure(
                f"command did not return one JSON object: {command}: {error}"
            ) from error
        if not isinstance(payload, dict):
            raise AcceptanceFailure(f"command returned a non-object JSON value: {command}")
        result["payload"] = payload
    else:
        result["stdout_tail"] = stdout[-1000:]
        result["stderr_tail"] = stderr[-1000:]
    return result


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AcceptanceFailure(message)


def loop_graph(run_id: str) -> dict[str, Any]:
    retry_policy = {
        "max_attempts": 3,
        "initial_backoff_seconds": 1,
        "backoff_multiplier": 2,
        "max_backoff_seconds": 8,
        "identical_failure_limit": 2,
    }
    return {
        "schema_version": 1,
        "run_id": run_id,
        "revision": 1,
        "previous_revision_digest": None,
        "state": "active",
        "terminal_reason": None,
        "entry_nodes": ["work"],
        "required_criteria": ["task-pass", "judge-pass"],
        "passed_criteria": [],
        "nodes": [
            {
                "id": "work",
                "executor_role_id": "task-agent",
                "verifier_role_id": "test-verifier",
                "criterion_ids": ["task-pass"],
                "completion_predicates": [
                    {"evidence_id": "test-pass", "kind": "artifact", "result": "present"}
                ],
                "required_capabilities": [],
                "retry_policy": retry_policy,
            },
            {
                "id": "judge",
                "executor_role_id": "judge-agent",
                "verifier_role_id": "judge-verifier",
                "criterion_ids": ["judge-pass"],
                "completion_predicates": [
                    {
                        "evidence_id": "judge-receipt",
                        "kind": "artifact",
                        "result": "present",
                    }
                ],
                "required_capabilities": [],
                "retry_policy": retry_policy,
            },
        ],
        "edges": [
            {
                "id": "work-to-judge",
                "from": "work",
                "to": "judge",
                "predicates": [
                    {
                        "evidence_id": "verify-work",
                        "kind": "independent-verification",
                        "result": "passed",
                    }
                ],
            }
        ],
        "evidence": [],
        "attempts": [],
        "capability_support": [],
        "steering": [],
    }


def run_status(run_id: str, *, state: str, cancel_requested: bool) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "run_id": run_id,
        "revision": 1,
        "state": state,
        "required_criteria": ["task-pass", "judge-pass"],
        "passed_criteria": [],
        "failed_criteria": [],
        "blocked_criteria": [],
        "active_roles": ["task-agent", "judge-agent"],
        "next_action": None if state == "cancelled" else "retry the disposable test",
        "latest_evidence": [],
        "blocker": None,
        "updated_at": "2026-08-24T00:00:00Z",
        "host": "codex",
        "host_version": "disposable-acceptance",
        "surface": "app",
        "external_runtime": None,
        "resolved_owner": "host-native",
        "resolution_evidence_digest": digest_bytes(b"disposable-owner"),
        "subagent_support": "supported",
        "resume_note": None,
        "criterion_evidence": {},
        "continuation": {
            "session_binding_digest": digest_bytes(b"session-before-compaction"),
            "max_retry_attempts": 3,
            "attempts_used": 1,
            "cancel_requested": cancel_requested,
        },
    }


def copy_judge_fixture(target: Path) -> None:
    judge = target / "judge"
    judge.mkdir(parents=True, exist_ok=True)
    for fixture in JUDGE_FIXTURES.glob("*.json"):
        shutil.copyfile(fixture, judge / fixture.name)
    files = {
        "artifact/patch.diff": b"diff --git a/src b/src\n",
        "evidence/tests.json": b'{"passed":18,"failed":0}\n',
        "artifact/activation.diff": b"security activation patch\n",
        "evidence/hostile-tests.json": b'{"passed":9,"failed":0}\n',
    }
    for relative, content in files.items():
        path = target / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)


def main() -> int:
    parser = argparse.ArgumentParser()
    default_binary = ROOT / "target/debug" / ("hive.exe" if sys.platform == "win32" else "hive")
    parser.add_argument("--hive-bin", type=Path, default=default_binary)
    parser.add_argument("--work-parent", type=Path, default=ROOT / "tests/work")
    arguments = parser.parse_args()

    binary = arguments.hive_bin.resolve()
    require(binary.is_file(), f"Hive binary does not exist: {binary}")
    work_parent = arguments.work_parent.resolve()
    work_parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="verified-workflow-acceptance-", dir=work_parent))
    target = work / "consumer"
    run_id = "disposable-run"
    run = target / ".hive/runs" / run_id
    run.mkdir(parents=True)
    plan = (
        "# Disposable verified workflow\n\n"
        "- [ ] task-pass: intentional test succeeds after bounded retry\n"
        "- [ ] judge-pass: independent Judge receipt is verified\n"
    )
    (run / "PLAN.md").write_text(plan, encoding="utf-8", newline="\n")

    receipt: dict[str, Any] = {
        "schema_version": 1,
        "acceptance": "verified-workflow-disposable",
        "started_at": datetime.now(UTC).isoformat(),
        "status": "running",
        "work_directory": str(work),
        "hive_binary": str(binary),
        "stable_release_attempted": False,
        "provider_processes_started": 0,
        "steps": [],
        "limitations": [
            "The host converts natural language into normalized routing facts; hive route does not ingest raw prompts.",
            "Fresh-process canonical recovery is tested; the Codex desktop application itself is not restarted.",
            "A host-owned Judge verdict receipt is verified, while authenticated quorum completion authority remains a separate protected-trust-root boundary.",
        ],
    }

    try:
        natural_request = (
            "Create a bounded run, retry one intentional failure, require an independent Judge, "
            "recover after compaction, and honor user cancellation."
        )
        request_path = work / "natural-request.txt"
        request_path.write_text(natural_request + "\n", encoding="utf-8", newline="\n")
        route_request = json.loads(ROUTE_FIXTURE.read_text(encoding="utf-8"))
        route_path = work / "routing-request.json"
        write_json(route_path, route_request)
        routed = invoke(
            [str(binary), "route", "--request", str(route_path), "--output", "json"],
            cwd=ROOT,
            expect_json=True,
        )
        route_payload = routed["payload"]
        require(routed["return_code"] == 0, "natural-language routing facts were rejected")
        require(route_payload.get("data", {}).get("selected_skill") == "verified-workflow", "verified-workflow was not selected")
        receipt["steps"].append(
            {
                "name": "natural-language-routing",
                "status": "passed",
                "request_digest": digest_bytes(request_path.read_bytes()),
                "normalized_signals": route_request["workflow_signals"],
                "process_id": routed["process_id"],
                "selected_skill": "verified-workflow",
            }
        )

        graph_path = work / "initial-loop.md"
        graph_path.write_bytes(markdown_document(loop_graph(run_id), "# Disposable loop\n"))
        initialized = invoke(
            [str(binary), "loop", "initialize", "--target", str(target), "--graph", str(graph_path), "--output", "json"],
            cwd=ROOT,
            expect_json=True,
        )
        require(initialized["return_code"] == 0, "loop initialization failed")
        validated = invoke(
            [str(binary), "loop", "validate", "--target", str(target), "--run", run_id, "--output", "json"],
            cwd=ROOT,
            expect_json=True,
        )
        require(validated["return_code"] == 0, "loop validation failed")
        graph_digest = initialized["payload"]["data"]["graph_digest"]
        require(validated["payload"]["data"]["graph_digest"] == graph_digest, "initialized graph digest changed during validation")
        receipt["steps"].append(
            {
                "name": "run-initialize-validate",
                "status": "passed",
                "initialize_process_id": initialized["process_id"],
                "validate_process_id": validated["process_id"],
                "run_id": run_id,
                "graph_digest": graph_digest,
            }
        )

        task = work / "task"
        task.mkdir()
        implementation = task / "implementation.py"
        implementation.write_text("def answer():\n    return 1\n", encoding="utf-8", newline="\n")
        (task / "test_task.py").write_text(
            "import unittest\nfrom implementation import answer\n\n"
            "class TaskTest(unittest.TestCase):\n"
            "    def test_expected_result(self):\n"
            "        self.assertEqual(answer(), 2)\n\n"
            "if __name__ == '__main__':\n    unittest.main()\n",
            encoding="utf-8",
            newline="\n",
        )
        first_attempt = invoke([sys.executable, "-B", "-m", "unittest", "-v", "test_task.py"], cwd=task)
        require(first_attempt["return_code"] != 0, "the intentional first attempt did not fail")

        (run / "STATUS.md").write_bytes(markdown_document(run_status(run_id, state="executing", cancel_requested=False), "# Run status\n"))
        retry_gate = invoke(
            [str(binary), "run", "closure", "--target", str(target), "--run", run_id, "--output", "json"],
            cwd=ROOT,
            expect_json=True,
        )
        require(retry_gate["return_code"] == 0, "retry closure gate failed")
        continuation = retry_gate["payload"]["data"]["continuation"]
        require(continuation["retry_budget"]["retry_permitted"] is True, "bounded retry was not permitted")
        require(continuation["retry_budget"]["remaining_attempts"] == 2, "bounded retry count is wrong")

        implementation.write_text("def answer():\n    return 2\n", encoding="utf-8", newline="\n")
        shutil.rmtree(task / "__pycache__", ignore_errors=True)
        second_attempt = invoke([sys.executable, "-B", "-m", "unittest", "-v", "test_task.py"], cwd=task)
        require(second_attempt["return_code"] == 0, "the bounded retry did not succeed")
        receipt["steps"].append(
            {
                "name": "intentional-failure-bounded-retry",
                "status": "passed",
                "attempts": [
                    {"attempt": 1, "outcome": "failed", "process_id": first_attempt["process_id"], "stderr_digest": first_attempt["stderr_digest"]},
                    {"attempt": 2, "outcome": "succeeded", "process_id": second_attempt["process_id"], "stderr_digest": second_attempt["stderr_digest"]},
                ],
                "remaining_attempts_before_retry": 2,
                "task_artifact_digest": digest_bytes(implementation.read_bytes()),
            }
        )

        copy_judge_fixture(target)
        judge = invoke(
            [
                str(binary), "judge", "receipt", "--target", str(target),
                "--request", "judge/host-receipt-codex.json",
                "--package", "judge/package-elevated.json",
                "--assignment", "judge/assignment-elevated.json",
                "--output", "json",
            ],
            cwd=ROOT,
            expect_json=True,
        )
        require(judge["return_code"] == 0, "independent Judge receipt was rejected")
        judge_data = judge["payload"]["data"]
        judge_fixture = json.loads((JUDGE_FIXTURES / "host-receipt-codex.json").read_text(encoding="utf-8"))
        require(judge_fixture["judge_instance_id"] != judge_fixture["task_agent_id"], "Judge identity is not independent")
        require(judge_data["spawned"] is False, "Hive unexpectedly spawned the Judge")
        require(judge_data["completion_authority"] is False, "a single Judge receipt gained completion authority")
        require(judge_data["quorum_required"] is True, "Judge quorum was not required")
        receipt["steps"].append(
            {
                "name": "independent-judge",
                "status": "passed",
                "process_id": judge["process_id"],
                "task_agent_id": judge_fixture["task_agent_id"],
                "judge_instance_id": judge_fixture["judge_instance_id"],
                "host": judge_fixture["host"],
                "model_id": judge_fixture["launch"]["model_id"],
                "completion_authority": False,
                "quorum_required": True,
                "spawned": False,
            }
        )

        recovered = invoke(
            [str(binary), "loop", "recover", "--target", str(target), "--run", run_id, "--output", "json"],
            cwd=ROOT,
            expect_json=True,
        )
        fresh_session = invoke(
            [
                str(binary), "run", "continuation", "--target", str(target),
                "--run", run_id, "--session-id", "session-after-compaction", "--output", "json",
            ],
            cwd=ROOT,
            expect_json=True,
        )
        require(recovered["return_code"] == 0, "fresh-process recovery failed")
        require(fresh_session["return_code"] == 0, "fresh-session continuation check failed")
        recovery_data = recovered["payload"]["data"]
        session_data = fresh_session["payload"]["data"]
        require(recovery_data["graph_digest"] == graph_digest, "recovered graph digest differs")
        require(recovery_data["recovered_from"] == "canonical-markdown", "recovery did not use canonical Markdown")
        require(recovered["process_id"] not in {initialized["process_id"], validated["process_id"]}, "recovery reused an earlier process")
        require(session_data["decision"] == "allow", "fresh session was not admitted safely")
        require(session_data["reason"] == "session-binding-mismatch", "fresh session identity was not distinguished")
        receipt["steps"].append(
            {
                "name": "compaction-restart-recovery",
                "status": "passed",
                "old_session_id": "session-before-compaction",
                "new_session_id": "session-after-compaction",
                "recovery_process_id": recovered["process_id"],
                "continuation_process_id": fresh_session["process_id"],
                "session_decision": session_data["decision"],
                "session_reason": session_data["reason"],
                "recovered_from": recovery_data["recovered_from"],
                "graph_digest": recovery_data["graph_digest"],
            }
        )

        (run / "STATUS.md").write_bytes(markdown_document(run_status(run_id, state="cancelled", cancel_requested=True), "# Run status\n"))
        cancelled = invoke(
            [str(binary), "run", "closure", "--target", str(target), "--run", run_id, "--output", "json"],
            cwd=ROOT,
            expect_json=True,
        )
        cancelled_continuation = invoke(
            [
                str(binary), "run", "continuation", "--target", str(target),
                "--run", run_id, "--session-id", "session-after-compaction", "--output", "json",
            ],
            cwd=ROOT,
            expect_json=True,
        )
        require(cancelled["return_code"] == 0, "cancelled run closure failed")
        require(cancelled_continuation["return_code"] == 0, "cancelled continuation check failed")
        cancelled_data = cancelled["payload"]["data"]
        cancelled_gate = cancelled_continuation["payload"]["data"]
        require(cancelled_data["closure"]["ready_for_final"] is True, "cancelled run did not become terminal")
        require(cancelled_data["continuation"]["cancel"]["state"] == "cancelled", "user cancellation was not retained")
        require(cancelled_data["continuation"]["retry_budget"]["retry_permitted"] is False, "retry remained permitted after cancellation")
        require(cancelled_data["continuation"]["spawned"] is False, "cancellation spawned a process")
        require(cancelled_gate["decision"] == "allow", "cancelled continuation was not allowed to stop")
        require(cancelled_gate["nudge_claimed"] is False, "cancelled continuation claimed a nudge")
        require(cancelled_gate["spawned"] is False, "cancelled continuation spawned a process")
        receipt["steps"].append(
            {
                "name": "user-cancellation",
                "status": "passed",
                "process_id": cancelled["process_id"],
                "continuation_process_id": cancelled_continuation["process_id"],
                "cancel_state": "cancelled",
                "continuation_decision": cancelled_gate["decision"],
                "continuation_reason": cancelled_gate["reason"],
                "nudge_claimed": False,
                "retry_permitted": False,
                "ready_for_final": True,
                "spawned": False,
            }
        )

        receipt["status"] = "passed"
    except (AcceptanceFailure, KeyError, subprocess.TimeoutExpired) as error:
        receipt["status"] = "failed"
        receipt["failure"] = str(error)
    finally:
        receipt["finished_at"] = datetime.now(UTC).isoformat()
        receipt_path = work / "acceptance-receipt.json"
        receipt["receipt_path"] = str(receipt_path)
        write_json(receipt_path, receipt)
        print(json.dumps(receipt, ensure_ascii=False, indent=2))

    return 0 if receipt["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
