"""Small setup inputs and results use the real CLI in disposable profiles."""
import json
import os
import subprocess

from jsonschema import Draft202012Validator
from tests.conformance.support.harness import (
    ACTION_RESULT_SCHEMA, Phase1CliTestCase, read_yaml, snapshot_tree, write_yaml,
)


class SetupSummaryConformance(Phase1CliTestCase):
    def setup_result(self, *flags, expected=0, limit=8192):
        process = subprocess.run(
            [str(self.hive_binary), "setup", *map(str, flags), "--user-root",
             str(self.setup_user_root), "--output", "json"],
            input="", capture_output=True, text=True, encoding="utf-8", timeout=30,
            env={**os.environ, "PATH": str(self.work_root / "no-external-tools")},
        )
        self.assertEqual(process.returncode, expected, (process.stdout, process.stderr))
        self.assertLessEqual(len(process.stdout.encode("utf-8")), limit)
        result = json.loads(process.stdout)
        Draft202012Validator(ACTION_RESULT_SCHEMA).validate(result)
        self.assertEqual(result["exit_code"], process.returncode)
        return result

    def test_user_status_and_one_localized_legacy_question_are_read_only(self):
        config = self.setup_user_root / ".hive/config/user-setup.yml"
        saved = read_yaml(config)
        saved["interface_language"] = "ko"
        write_yaml(config, saved)
        before = snapshot_tree(self.setup_user_root)
        status = self.setup_result("--scope", "user", "--show", "--summary", limit=4096)
        self.assertTrue(status["data"]["configured"])
        self.assertEqual(status["data"]["settings"]["interface_language"], "ko")
        self.assertFalse(status["data"]["authorizes_dispatch"])
        self.assertGreater(status["data"]["source_counts"]["legacy-unknown"], 0)
        question = self.setup_result("--scope", "user", "--describe", "--question", "usage-enforcement")
        entries = question["data"]["questions"]
        self.assertEqual(len(entries), 1)
        self.assertTrue(entries[0]["current"])
        self.assertEqual(entries[0]["source"], "legacy-unknown")
        self.assertIsInstance(entries[0]["question"]["prompt"], str)
        self.assertTrue(any("가" <= c <= "힣" for c in entries[0]["question"]["prompt"]))
        self.assertEqual(snapshot_tree(self.setup_user_root), before)

    def test_project_patch_preserves_unspecified_choices_and_repeated_input_is_idempotent(self):
        target = self.work_root / "consumer"
        target.mkdir()
        process, _ = self.invoke_setup(target, extra_arguments=("--summary",))
        self.assertEqual(process.returncode, 0, process.stdout)
        self.assertLessEqual(len(process.stdout.encode("utf-8")), 8192)
        settings = target / ".hive/setup-answers.yml"
        original = read_yaml(settings)
        before = snapshot_tree(target)
        user_settings = self.setup_user_root / ".hive/config/user-setup.yml"
        user_before = user_settings.read_bytes()
        status = self.setup_result("--target", target, "--show", "--summary", limit=4096)
        self.assertEqual(status["data"]["settings"]["primary_host"], "codex")
        question = self.setup_result("--target", target, "--describe", "--question", "project-name")
        self.assertEqual(question["data"]["questions"][0]["current"], original["project_name"])
        self.assertEqual(snapshot_tree(target), before)
        patch = self.work_root / "patch.json"
        patch.write_text(json.dumps({"schema_version": 1, "kind": "hive.setup-answer-patch",
            "answers": {"project_name": "Renamed consumer"}}), encoding="utf-8")
        applied, result = self.invoke_setup(target, answers=patch,
            extra_arguments=("--input-channel", "agent", "--summary"))
        self.assertEqual(applied.returncode, 0, applied.stdout)
        self.assertTrue(result["data"]["completion_verified"])
        self.assertLessEqual(len(applied.stdout.encode("utf-8")), 8192)
        changed = read_yaml(settings)
        self.assertEqual(changed["project_name"], "Renamed consumer")
        for key, value in original.items():
            if key not in ("project_name", "preference_decisions"):
                self.assertEqual(changed[key], value, key)
        for key, value in original["preference_decisions"].items():
            if key != "/project_name":
                self.assertEqual(changed["preference_decisions"][key], value, key)
        decision = changed["preference_decisions"]["/project_name"]
        self.assertEqual(decision["origin"], "external-change")
        self.assertIsNone(decision["decided_at"])
        bytes_before = settings.read_bytes()
        repeated, _ = self.invoke_setup(target, answers=patch,
            extra_arguments=("--input-channel", "agent", "--summary"))
        self.assertEqual(repeated.returncode, 0, repeated.stdout)
        self.assertEqual(settings.read_bytes(), bytes_before)
        # Project journals and the shared derived index are allowed to advance.
        self.assertEqual(user_settings.read_bytes(), user_before)
        validation, valid = self.invoke_setup(target, answers=patch, mode="--validate",
            extra_arguments=("--input-channel", "agent", "--summary"))
        self.assertEqual(validation.returncode, 0, validation.stdout)
        self.assertFalse(valid["data"]["completion_verified"])
        self.assertEqual(settings.read_bytes(), bytes_before)
        # A lost AI response is resolved by reading state, without repeating application.
        after = snapshot_tree(target)
        self.setup_result("--target", target, "--show", "--summary", limit=4096)
        self.assertEqual(snapshot_tree(target), after)

    def test_patch_metadata_and_missing_initial_selectors_never_create_settings(self):
        patch = self.work_root / "patch.json"
        before = snapshot_tree(self.setup_user_root)
        for answers in ({"preference_decisions": {}}, {"operation_id": "foreign"}):
            with self.subTest(answers=answers):
                patch.write_text(json.dumps({"schema_version": 1, "kind": "hive.setup-answer-patch",
                    "answers": answers}), encoding="utf-8")
                self.setup_result("--scope", "user", "--answers", patch, "--apply", "--summary", expected=2)
                self.assertEqual(snapshot_tree(self.setup_user_root), before)
        target = self.work_root / "empty-consumer"
        target.mkdir()
        patch.write_text(json.dumps({"schema_version": 1, "kind": "hive.setup-answer-patch",
            "answers": {"project_name": "Incomplete"}}), encoding="utf-8")
        process, _ = self.invoke_setup(target, answers=patch, extra_arguments=("--summary",))
        self.assertEqual(process.returncode, 2, process.stdout)
        self.assertEqual(snapshot_tree(target), {})
        self.assertEqual(snapshot_tree(self.setup_user_root), before)

    def test_summary_rejects_condensed_approval_preview(self):
        before = snapshot_tree(self.setup_user_root)
        result = self.setup_result("--scope", "user", "--dry-run", "--summary", expected=2)
        self.assertEqual(result["code"], "hive.invalid-input")
        self.assertEqual(snapshot_tree(self.setup_user_root), before)
