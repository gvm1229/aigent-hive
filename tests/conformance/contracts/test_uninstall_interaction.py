"""Removal consent and recovery use private files; no installed program or OS service is removed."""
import json
import os
import subprocess

from jsonschema import Draft202012Validator
from tests.conformance.support.harness import ACTION_RESULT_SCHEMA, Phase1CliTestCase, snapshot_tree


class UninstallInteractionConformance(Phase1CliTestCase):
    def invoke(self, root, *flags, environment=None):
        process = subprocess.run([str(self.hive_binary), "uninstall", "--user-root", str(root), *flags, "--output", "json"],
            input="yes\n예\n", capture_output=True, text=True, encoding="utf-8", timeout=30,
            env={**os.environ, "PATH": str(self.work_root / "no-external-commands"), **(environment or {})})
        if process.returncode == 90:
            return process, None
        result = json.loads(process.stdout)
        Draft202012Validator(ACTION_RESULT_SCHEMA).validate(result)
        self.assertEqual(result["exit_code"], process.returncode)
        return process, result

    def test_preview_and_unapproved_json_preserve_every_file_and_never_take_pipe_consent(self):
        root = self.setup_user_root
        knowledge = root / ".hive/knowledge/note.md"
        knowledge.parent.mkdir(parents=True)
        knowledge.write_text("Preserved user knowledge.", encoding="utf-8")
        before = snapshot_tree(root)
        for flags, code, exit_code in [((), "hive.uninstall-needs-input", 2), (("--dry-run",), "hive.uninstall-preview", 0)]:
            process, result = self.invoke(root, *flags)
            self.assertEqual(process.returncode, exit_code, result)
            self.assertEqual(result["code"], code)
            self.assertEqual(result["changed_paths"], [])
            self.assertIn(".hive/knowledge", result["data"]["preview"]["preserved"])
            self.assertEqual(snapshot_tree(root), before)

    def test_confirmed_private_file_removal_resumes_each_exit_without_claiming_unknown_program_removed(self):
        for boundary in ["approved", "monitor-stopped", "hosts-removed", "files-removed-before-checkpoint"]:
            with self.subTest(boundary=boundary):
                root = self.work_root / boundary
                derived = root / ".hive/index/derived.db"
                derived.parent.mkdir(parents=True)
                derived.write_bytes(b"disposable index")
                knowledge = root / ".hive/knowledge/note.md"
                knowledge.parent.mkdir(parents=True)
                knowledge.write_bytes(b"user knowledge")
                projects = root / ".hive/config/projects.yml"
                projects.parent.mkdir(parents=True)
                projects.write_bytes(b"preserved registry")
                interrupted, _ = self.invoke(root, "--confirm", environment={"HIVE_TEST_UNINSTALL_EXIT_AFTER": boundary})
                self.assertEqual(interrupted.returncode, 90, (interrupted.stdout, interrupted.stderr))
                receipts = list((root / ".hive/history/lifecycle").glob("uninstall-*.json"))
                self.assertEqual(len(receipts), 1)
                receipt = json.loads(receipts[0].read_text(encoding="utf-8"))
                process, result = self.invoke(root, "--resume", receipt["operation_id"])
                self.assertEqual(process.returncode, 2, result)
                self.assertEqual(result["code"], "hive.user-uninstall-pending")
                self.assertFalse(result["data"]["completion_verified"])
                self.assertEqual(result["data"]["core"]["owner"], "unverified")
                self.assertEqual(result["data"]["steps"]["files"], "removed")
                self.assertEqual(result["data"]["steps"]["core"], "unverified")
                self.assertEqual(result["data"]["operation_id"], receipt["operation_id"])
                self.assertFalse(derived.exists())
                self.assertEqual(knowledge.read_bytes(), b"user knowledge")
                self.assertEqual(projects.read_bytes(), b"preserved registry")
                self.assertTrue(receipts[0].exists())
                self.assertFalse((root / ".hive/maintenance").exists())
