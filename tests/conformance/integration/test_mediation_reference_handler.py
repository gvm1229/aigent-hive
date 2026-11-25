"""Host-owned reference handler round trip over the file inbox with a fake worker.

The test plays the interactive host: it claims a message, checks the published worker policy,
launches a tiny fake worker itself and acknowledges with the claim token. Hive must only record
state; sentinel `codex`, `claude` and `gemini` executables on PATH prove it launches nothing.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import unittest

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[3]
SCHEMA = json.loads((ROOT / "schemas/action-result.schema.json").read_text(encoding="utf-8"))
FAKE_WORKER = """\
import json, sys
from pathlib import Path
policy = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))["policy"]
Path(sys.argv[2]).write_text(
    "fake worker result\\nsandbox=" + policy["sandbox"] + "\\n", encoding="utf-8"
)
Path(sys.argv[3]).write_text("ran\\n", encoding="utf-8")
"""
SENTINEL = """#!/bin/sh
echo "$0 $*" >> "{marker}"
exit 97
"""


class MediationReferenceHandler(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="mediation-handler-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.target = self.root / "consumer"
        self.home = self.root / "home"
        (self.target / ".hive").mkdir(parents=True)
        self.home.mkdir()
        self.binary = os.environ.get(
            "HIVE_BIN", str(ROOT / "target/debug" / ("hive.exe" if os.name == "nt" else "hive"))
        )
        self.marker = self.root / "host-cli-launched.txt"
        self.environment = dict(os.environ)
        if os.name != "nt":
            shims = self.root / "shims"
            shims.mkdir()
            for name in ("codex", "claude", "gemini"):
                shim = shims / name
                shim.write_text(SENTINEL.format(marker=self.marker), encoding="utf-8")
                shim.chmod(shim.stat().st_mode | stat.S_IXUSR)
            self.environment["PATH"] = f"{shims}{os.pathsep}{self.environment.get('PATH', '')}"

    def hive(self, *arguments: str) -> dict:
        process = subprocess.run(
            [self.binary, "msg", *arguments, "--target", str(self.target),
             "--user-root", str(self.home), "--output", "json"],
            capture_output=True, text=True, encoding="utf-8", timeout=30, env=self.environment,
        )
        value = json.loads(process.stdout)
        Draft202012Validator(SCHEMA).validate(value)
        self.assertEqual(process.returncode, value["exit_code"], value)
        self.assertEqual(value["action"], "Message")
        return value

    def ack_supports_claim(self) -> bool:
        process = subprocess.run(
            [self.binary, "msg", "ack", "--help"],
            capture_output=True, text=True, encoding="utf-8", timeout=30, env=self.environment,
        )
        usage = [line for line in process.stdout.splitlines() if "hive msg ack" in line]
        return any("--claim" in line for line in usage)

    def test_host_claims_runs_fake_worker_under_policy_and_acknowledges_once(self) -> None:
        body = self.root / "brief.txt"
        body.write_text("Implement the change in the task worktree.\n", encoding="utf-8")
        sent = self.hive("send", "--to", "hephaistos", "--from", "athena",
                         "--file", str(body), "--task", "task-1")
        self.assertEqual(sent["code"], "hive.msg-queued", sent)
        self.assertFalse(sent["data"]["waited"])
        message = sent["data"]["message"]
        message_id = message["id"]
        self.assertEqual(message["body"]["trust"], "untrusted-data")

        claimed = self.hive("claim", "--id", message_id, "--role", "hephaistos")
        self.assertEqual(claimed["code"], "hive.msg-claimed", claimed)
        token = claimed["data"]["claim_token"]
        self.assertTrue(token)
        self.assertFalse(claimed["data"]["waited"])

        policy_path = self.target / ".hive/mediation/policies" / f"{message_id}.json"
        record = json.loads(policy_path.read_text(encoding="utf-8"))
        self.assertEqual(record["message_id"], message_id)
        policy = record["policy"]
        self.assertEqual(policy["sandbox"], "workspace-write")
        self.assertEqual(policy["network"], "none")
        self.assertEqual(policy["git_push"], "deny")
        self.assertEqual(policy["writable_roots"], ["{target}"])
        self.assertEqual(policy["git_env"]["GIT_CONFIG_VALUE_0"], "never")
        self.assertIs(policy["hive_launches_worker"], False)
        self.assertEqual(policy["role"], message["recipient_role"])

        # The host, not Hive, launches the worker under the policy.
        worker = self.root / "fake_worker.py"
        worker.write_text(FAKE_WORKER, encoding="utf-8")
        result = self.root / "result.txt"
        ran = self.root / "fake-worker-ran.txt"
        subprocess.run([sys.executable, "-I", str(worker), str(policy_path), str(result), str(ran)],
                       check=True, timeout=30, env=self.environment)
        self.assertEqual(ran.read_text(encoding="utf-8"), "ran\n")

        ack = ["ack", "--id", message_id, "--result", str(result)]
        if self.ack_supports_claim():
            ack += ["--claim", token]
        first = self.hive(*ack)
        self.assertEqual(first["code"], "hive.msg-acknowledged", first)
        second = self.hive(*ack)
        self.assertEqual(second["code"], "hive.msg-duplicate", second)

        status = self.hive("status", "--id", message_id)
        self.assertEqual(status["data"]["message"]["state"], "acknowledged", status)
        self.assertFalse(status["data"]["message"]["auto_redispatch"])
        receipts = list((self.target / ".hive/mediation/receipts").glob(f"{message_id}*"))
        self.assertEqual(len(receipts), 1, receipts)

        self.assertFalse(self.marker.exists(), "Hive launched a host CLI")


if __name__ == "__main__":
    unittest.main()
