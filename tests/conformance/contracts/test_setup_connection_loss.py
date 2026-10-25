"""Simulated AI delivery failures cannot own the private user's CLI conversation."""
import json
import os
from pathlib import Path
import subprocess
import time
import unittest

from tests.conformance.contracts.test_setup_console_interrupt import compile_console_fixture
from tests.conformance.support.harness import FIXTURE_ROOT, Phase1CliTestCase, read_yaml, snapshot_tree


@unittest.skipUnless(os.name == "nt", "this fixture needs an independently owned Windows console")
class SetupConnectionLossConformance(Phase1CliTestCase):
    def inspect(self, target):
        result = subprocess.run(
            [str(self.hive_binary), "setup", "--target", str(target), "--user-root", str(self.setup_user_root),
             "--show", "--summary", "--output", "json"],
            input="", capture_output=True, text=True, encoding="utf-8", timeout=15,
        )
        self.assertEqual(result.returncode, 0, (result.stdout, result.stderr))
        self.assertLessEqual(len(result.stdout.encode("utf-8")), 4096)
        return json.loads(result.stdout)["data"]

    def lose_ai_reply(self, stage, result):
        # The connection returns no response. It has no access to the user's console process.
        with self.assertRaisesRegex(ConnectionError, stage):
            if result is not None:
                raise ConnectionError(stage)

    def test_loss_before_questions_during_questions_and_after_apply_never_reapplies(self):
        helper = compile_console_fixture(self.work_root)
        target = self.work_root / "consumer"
        target.mkdir()
        controls = self.work_root / "connection"
        controls.mkdir()
        user_config = self.setup_user_root / ".hive/config/user-setup.yml"
        user_before = user_config.read_bytes()
        initial_tree = snapshot_tree(target)
        first = self.inspect(target)
        self.lose_ai_reply("before-cli-start", first)
        reconnected = self.inspect(target)
        self.assertFalse(reconnected["configured"])
        self.assertIsNone(reconnected["pending"])
        self.assertEqual(snapshot_tree(target), initial_tree)
        self.assertEqual(user_config.read_bytes(), user_before)

        result_path = controls / "completed.json"
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = subprocess.SW_HIDE
        # Exactly one CLI conversation, in its own console, with fixed user answers.
        helper_process = subprocess.Popen(
            [str(helper), str(self.hive_binary), str(self.setup_user_root), str(result_path),
             "connection-loss", str(target), str(FIXTURE_ROOT / "capabilities-codex-host-native.json"), str(controls)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
            creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup,
        )
        try:
            ready = controls / "question.ready"
            deadline = time.monotonic() + 20
            while not ready.exists() and helper_process.poll() is None and time.monotonic() < deadline:
                time.sleep(0.025)
            self.assertTrue(ready.exists(), "private CLI did not reach the second question")
            progress_path = Path(ready.read_text())
            self.assertTrue(progress_path.resolve().is_relative_to(self.setup_user_root.resolve()))
            pending = read_yaml(progress_path)
            self.assertEqual(pending["step"], "project-name")
            self.assertIn("project-mode", pending["answered_questions"])
            self.assertNotIn("project-name", pending["answered_questions"])
            during = self.inspect(target)
            self.lose_ai_reply("while-cli-question-waits", during)
            self.assertIsNone(helper_process.poll())
            self.assertEqual(self.inspect(target)["pending"]["operation_id"], pending["operation_id"])
            self.assertEqual(read_yaml(progress_path), pending)
            self.assertFalse((target / ".hive/setup-answers.yml").exists())
            (controls / "continue-user").write_bytes(b"fixed user answers may continue\n")
            stdout, stderr = helper_process.communicate(timeout=45)
            self.assertEqual(helper_process.returncode, 0, (stdout, stderr))
        finally:
            if helper_process.poll() is None:
                # Let the fixed helper close its own child; no unbounded orphan on failed assertions.
                (controls / "continue-user").write_bytes(b"fixture teardown\n")
                helper_process.communicate(timeout=45)

        completed = read_yaml(progress_path)
        self.assertEqual(completed["phase"], "complete")
        self.assertEqual(completed["operation_id"], pending["operation_id"])
        self.assertEqual(read_yaml(target / ".hive/setup-answers.yml")["project_name"], "Connection loss fixture")
        self.lose_ai_reply("after-apply-before-delivery", json.loads(result_path.read_text()))
        applied_tree = snapshot_tree(target)
        root_tree = snapshot_tree(self.setup_user_root)
        after = self.inspect(target)
        self.assertTrue(after["configured"])
        self.assertIsNone(after["pending"])
        self.assertFalse(after["authorizes_dispatch"])
        self.assertEqual(after["model_calls"], 0)
        self.assertEqual(snapshot_tree(target), applied_tree)
        self.assertEqual(snapshot_tree(self.setup_user_root), root_tree)
        self.assertEqual(user_config.read_bytes(), user_before)
