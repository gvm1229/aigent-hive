"""Small setup inputs and results use the real CLI in disposable profiles."""
import json
import os
import subprocess

from jsonschema import Draft202012Validator
from tests.conformance.support.harness import (
    ACTION_RESULT_SCHEMA, FIXTURE_ROOT, REPOSITORY_ROOT, Phase1CliTestCase, read_yaml, snapshot_tree, write_yaml,
)


class SetupSummaryConformance(Phase1CliTestCase):
    def test_minimal_new_project_patch_applies_shared_defaults_without_invented_choices(self):
        config = self.setup_user_root / ".hive/config/user-setup.yml"
        global_settings = read_yaml(config)
        global_settings["usage_guard"]["stop_remaining_percent"] = 37
        write_yaml(config, global_settings)
        global_before = config.read_bytes()
        for channel in ("agent", "answers-file"):
            with self.subTest(channel=channel):
                target = self.work_root / channel
                target.mkdir()
                (target / "AGENTS.md").write_bytes(b"User instructions.\r\n")
                (target / ".env").write_bytes(b"SYNTHETIC_VALUE=fixture\n")
                before = snapshot_tree(target)
                patch = self.work_root / (channel + ".json")
                patch.write_text(json.dumps({"schema_version": 1, "kind": "hive.setup-answer-patch",
                    "answers": {"setup_mode": "expedited", "project_name": "luna-quick",
                        "project_kind": "general", "primary_host": "codex"}}), encoding="utf-8")
                flags = ("--input-channel", channel)
                preview, _ = self.invoke_setup(target, answers=patch, mode="--dry-run", extra_arguments=flags)
                self.assertEqual(preview.returncode, 0, preview.stdout)
                self.assertEqual(snapshot_tree(target), before)
                applied, result = self.invoke_setup(target, answers=patch, extra_arguments=(*flags, "--summary"))
                self.assertEqual(applied.returncode, 0, applied.stdout)
                self.assertTrue(result["data"]["completion_verified"])
                settings = target / ".hive/setup-answers.yml"
                values = read_yaml(settings)
                self.assertEqual(values["usage_stop_remaining_percent"], 37)
                self.assertEqual(values["elevated_judge_quorum"], "2/3")
                self.assertEqual(values["critical_judge_quorum"], "3/3+human")
                for key in ("persistent_roles", "knowledge_include_paths", "approved_fallback_hooks",
                            "root_knowledge_promotion_categories", "approved_optional_skills"):
                    self.assertEqual(values[key], [], key)
                self.assertIn("**/.env.*", values["knowledge_exclude_paths"])
                self.assertIn("**/secrets/**", values["knowledge_exclude_paths"])
                records = values["preference_decisions"]
                self.assertEqual(records["/usage_stop_remaining_percent"]["origin"], "inherited")
                self.assertEqual(records["/elevated_judge_quorum"]["origin"], "default")
                self.assertEqual(records["/project_name"]["origin"],
                    "external-change" if channel == "agent" else "explicit")
                self.assertEqual(records["/project_name"]["input_channel"], channel)
                if channel == "agent":
                    self.assertIsNone(records["/project_name"]["decided_at"])
                else:
                    self.assertIsInstance(records["/project_name"]["decided_at"], int)
                self.assertIn(b"User instructions.\r\n", (target / "AGENTS.md").read_bytes())
                self.assertEqual((target / ".env").read_bytes(), b"SYNTHETIC_VALUE=fixture\n")
                saved = settings.read_bytes()
                for mode in ("--validate", "--apply"):
                    process, _ = self.invoke_setup(target, answers=patch, mode=mode, extra_arguments=flags)
                    self.assertEqual(process.returncode, 0, process.stdout)
                    self.assertEqual(settings.read_bytes(), saved)
                self.assertEqual(config.read_bytes(), global_before)

    def test_minimal_new_project_patch_resumes_without_answer_file_after_interruption(self):
        target = self.work_root / "interrupted"
        target.mkdir()
        note = target / "README.txt"
        note.write_bytes(b"Preserve this user file.\n")
        patch = self.work_root / "minimal.json"
        patch.write_text(json.dumps({"schema_version": 1, "kind": "hive.setup-answer-patch",
            "answers": {"setup_mode": "expedited", "project_name": "luna-quick",
                "project_kind": "general", "primary_host": "codex"}}), encoding="utf-8")
        interrupted = subprocess.run(
            [str(self.hive_binary), "setup", "--target", str(target), "--user-root",
             str(self.setup_user_root), "--answers", str(patch), "--capabilities",
             str(FIXTURE_ROOT / "capabilities-codex-host-native.json"), "--input-channel", "agent",
             "--apply", "--output", "json"], capture_output=True, text=True, timeout=30,
            env={**os.environ, "HIVE_PROJECT_SETUP_INTERRUPT_AFTER": "2"},
        )
        self.assertEqual(interrupted.returncode, 87, interrupted.stderr)
        progress_files = list((self.setup_user_root / ".hive/config/setup-progress/projects").glob("*.yml"))
        self.assertEqual(len(progress_files), 1)
        pending = read_yaml(progress_files[0])
        patch.unlink()
        resumed = self.setup_result("--target", target, "--resume", "--non-interactive", "--summary")
        self.assertTrue(resumed["data"]["completion_verified"])
        final = read_yaml(progress_files[0])
        self.assertEqual(final["operation_id"], pending["operation_id"])
        self.assertEqual(final["phase"], "complete")
        values = read_yaml(target / ".hive/setup-answers.yml")
        self.assertEqual(values["project_name"], "luna-quick")
        self.assertEqual(values["persistent_roles"], [])
        self.assertEqual(values["approved_fallback_hooks"], [])
        self.assertIsNone(values["preference_decisions"]["/project_name"]["decided_at"])
        self.assertEqual(note.read_bytes(), b"Preserve this user file.\n")

    def test_explicit_project_setup_skill_delivers_only_its_owned_companions(self):
        target = self.work_root / "selected-setup"
        target.mkdir()
        answers = read_yaml(FIXTURE_ROOT / "answers-no-role-no-hook.yml")
        answers.update({"setup_mode":"custom", "interface_language":"en",
            "wiki":{"enabled":False,"language":"en"}, "persona":{"id":"balanced"},
            "skills":{"mode":"individual","selected":["project-setup"]}})
        document = self.work_root / "answers.json"
        document.write_text(json.dumps(answers), encoding="utf-8")
        foreign = target / ".agents/skills/project-setup/references/foreign.md"
        foreign.parent.mkdir(parents=True)
        foreign.write_bytes(b"Unrelated user note.\r\n")
        for mode in ("--apply", "--validate"):
            process, result = self.invoke_setup(target, answers=document, mode=mode,
                extra_arguments=("--summary",))
            self.assertEqual(process.returncode, 0, (process.stdout, process.stderr))
            self.assertEqual(result["status"], "success")
            self.assertEqual(foreign.read_bytes(), b"Unrelated user note.\r\n")
            for name in ("advanced-setup.md", "directive-recovery.md", "structured-choices.md"):
                self.assertEqual((foreign.parent / name).read_bytes(),
                    (REPOSITORY_ROOT / "harness/skills/project-setup/references" / name).read_bytes())

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
        self.assertIn("language", status["data"]["available_sections"])
        self.assertIn("wiki", status["data"]["available_sections"])
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
        self.assertEqual(status["data"]["settings"]["project_name"], original["project_name"])
        self.assertEqual(status["data"]["settings"]["project_identity"], original["project_identity"])
        self.assertIn("identity", status["data"]["available_sections"])
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
