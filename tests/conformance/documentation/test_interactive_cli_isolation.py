"""No lifecycle mutation or private installation access in the on-host driver."""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("interactive_cli_isolation", ROOT / "scripts/qualify-interactive-cli.py")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class InteractiveCliIsolation(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.source = Path(self.temporary.name).resolve()
        self.binary = self.source / "target/hive.exe"
        self.binary.parent.mkdir()
        self.binary.write_bytes(b"fixture executable, never launched")
        self.fixture = self.source / "tests/work/inspection"
        self.inspection = MODULE.Inspection(self.fixture, self.binary, source=self.source)

    def test_private_paths_and_missing_tools_cannot_inherit_real_settings(self):
        with mock.patch.dict(os.environ, {"PATH": "personal-bin", "CODEX_HOME": "personal-host", "HIVE_USAGE_SENSOR": "real"}):
            environment = self.inspection.environment()
        for key in ("HOME", "USERPROFILE", "PATH", "CODEX_HOME", "NPM_CONFIG_PREFIX", "SystemRoot"):
            self.assertTrue(Path(environment[key]).is_relative_to(self.fixture))
        self.assertNotIn("HIVE_USAGE_SENSOR", environment)
        self.assertFalse(Path(environment["SystemRoot"]).exists())
        self.assertEqual(list(Path(environment["PATH"]).iterdir()), [])

    def test_external_install_and_personal_root_are_refused(self):
        for fixture, binary in ((self.source / "personal", self.binary),
                                (self.fixture, self.source.parent / "hive.exe")):
            with self.assertRaises(MODULE.IsolationError):
                MODULE.Inspection(fixture, binary, source=self.source)

    def test_mutating_driver_requires_isolated_os_and_confined_explicit_roots(self):
        import argparse
        options = argparse.Namespace(fixture=self.fixture, binary=self.binary,
            user_root=self.fixture / "user", result=self.fixture / "results.json", scenario="project", language="ko")
        with mock.patch.dict(os.environ, {"GITHUB_ACTIONS": "false"}):
            with self.assertRaises(MODULE.IsolationError):
                MODULE.lifecycle_command(options, source=self.source)
        self.assertFalse(self.fixture.exists())
        with mock.patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "RUNNER_ENVIRONMENT": "github-hosted"}):
            command = MODULE.lifecycle_command(options, source=self.source)
            self.assertIn(str(self.fixture / "user"), command)
            self.assertIn(str(self.fixture / "results.json"), command)
            options.user_root = self.source / "personal"
            with self.assertRaises(MODULE.IsolationError):
                MODULE.lifecycle_command(options, source=self.source)
            options.user_root = self.fixture / "user"
            options.result = self.source / "outside.json"
            with self.assertRaises(MODULE.IsolationError):
                MODULE.lifecycle_command(options, source=self.source)
        self.assertFalse(self.fixture.exists())

    def test_mutation_and_scope_override_are_refused_before_writes(self):
        for command in (["update", "--confirm"], ["uninstall", "--confirm"],
                        ["usage", "monitor", "enable"], ["setup", "--apply"],
                        ["setup", "--scope", "user", "--show", "--user-root", "personal"]):
            with self.assertRaises(MODULE.IsolationError), mock.patch.object(MODULE.subprocess, "run") as run:
                self.inspection.run(command)
            run.assert_not_called()
            self.assertFalse(self.fixture.exists())

    def test_inspection_passes_only_pinned_binary_private_scope_and_closed_input(self):
        with mock.patch.object(MODULE.subprocess, "run") as run:
            self.inspection.run(["setup", "--scope", "user", "--show"])
        args, kwargs = run.call_args
        self.assertEqual(args[0][0], str(self.binary))
        self.assertIn(str(self.inspection.user), args[0])
        self.assertEqual(kwargs["stdin"], MODULE.subprocess.DEVNULL)
        self.assertEqual(kwargs["env"]["PATH"], str(self.inspection.fake_bin))

    def test_link_escape_is_refused(self):
        self.fixture.mkdir(parents=True)
        outside = self.source / "personal"
        outside.mkdir()
        try:
            (self.fixture / "user").symlink_to(outside, target_is_directory=True)
        except OSError as error:
            if os.name != "nt":
                raise
            # A directory junction needs no symlink privilege and resolves identically.
            result = subprocess.run(
                [str(Path(os.environ["SystemRoot"]) / "System32/cmd.exe"),
                 "/d", "/c", "mklink", "/J", str(self.fixture / "user"), str(outside)],
                capture_output=True, check=False,
            )
            self.assertEqual(result.returncode, 0, (error, result.stderr))
        with self.assertRaises(MODULE.IsolationError):
            MODULE.Inspection(self.fixture, self.binary, source=self.source)


class ModelProjectReceipt(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("model_project_receipt",
            ROOT / "tests/fixtures/interactive-cli/model-project-receipt.py")
        self.recorder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.recorder)
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.trial = Path(temporary.name)
        (self.trial / "project").mkdir()
        (self.trial / "project/README.txt").write_bytes(b"Synthetic user note. Preserve exactly.\n")

    def test_same_case_limit_counts_failures_and_ignores_temporary_root_names(self):
        config = {"scenario": "project-quick", "model": "gpt-6-luna", "reasoning_effort": "medium",
                  "binary_sha256": "binary", "input_sha256": {"fixed-choices.json": "choices"}}
        renamed = {**config, "root": "another temporary root", "sample": 9, "source_commit": "docs-only"}
        self.assertEqual(self.recorder.case_identity(config), self.recorder.case_identity(renamed))
        reports = self.trial / "reports"
        reports.mkdir()
        for index in range(5):
            identity, previous = self.recorder.prior_case_runs(config, reports)
            self.assertEqual(len(previous), index)
            record = {"command": ["host:collaboration.spawn_agent"],
                      "status": "failed" if index < 4 else "running", "details": renamed}
            (reports / f"{index}.md").write_text("```json\n" + json.dumps(record) + "\n```\n", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "five runs"):
            self.recorder.prior_case_runs(config, reports)
        changed = {**config, "reasoning_effort": "low"}
        self.assertNotEqual(identity, self.recorder.case_identity(changed))
        self.assertEqual(self.recorder.prior_case_runs(changed, reports)[1], [])
        changed = {**config, "input_sha256": {"fixed-choices.json": "changed-choice"}}
        self.assertEqual(self.recorder.prior_case_runs(changed, reports)[1], [])

    def test_unpinned_new_model_input_cannot_reach_dispatch(self):
        with self.assertRaisesRegex(ValueError, "pinned"):
            self.recorder.prior_case_runs({"model": "gpt-6-luna"}, self.trial)

    def test_absent_final_settings_preserves_failed_events_and_measured_bytes(self):
        events = [{"operation": "hive", "arguments": ["setup", "--apply"],
                   "exit_code": 2, "hive_input_bytes": 13560}]
        (self.trial / "events.jsonl").write_text(json.dumps(events[0]) + "\n", encoding="utf-8")
        result = self.recorder.collect(self.trial, {"user_before": {}}, {"status": "completed"})
        self.assertEqual(result["events"], events)
        self.assertEqual(result["hive_input_bytes"], 13560)
        self.assertFalse(result["checks"]["final_settings_present"])
        self.assertFalse(result["checks"]["no_failed_cli_retry"])

    def test_missing_log_is_unknown_but_empty_observed_log_is_zero(self):
        absent = self.recorder.collect(self.trial, {"user_before": {}}, {})
        self.assertIsNone(absent["hive_input_bytes"])
        (self.trial / "events.jsonl").write_text("", encoding="utf-8")
        empty = self.recorder.collect(self.trial, {"user_before": {}}, {})
        self.assertEqual(empty["hive_input_bytes"], 0)
        self.assertFalse(empty["checks"]["host_completed"])

    def test_truncated_log_preserves_observed_prefix_without_claiming_a_total(self):
        (self.trial / "events.jsonl").write_text('{"hive_input_bytes":42}\n{', encoding="utf-8")
        result = self.recorder.collect(self.trial, {"user_before": {}}, {})
        self.assertEqual(result["observed_event_bytes"], 42)
        self.assertIsNone(result["hive_input_bytes"])
        self.assertEqual(len(result["events"]), 1)
        self.assertTrue(result["errors"])

    def test_user_case_checks_fixed_choices_preservation_and_same_recovery(self):
        import hashlib
        user = self.trial / "user"
        (user / ".hive/config").mkdir(parents=True)
        note = b"Synthetic user note. Preserve exactly.\n"
        (user / "README.txt").write_bytes(note)
        (user / ".hive/config/user-setup.yml").write_text(json.dumps({
            "interface_language": "ko", "usage_guard": {"enforcement_enabled": False},
            "preference_decisions": {"/interface_language": {"input_channel": "agent", "decided_at": None}},
        }), encoding="utf-8")
        (user / "progress.yml").write_text(json.dumps({"operation_id": "fixed", "phase": "complete"}), encoding="utf-8")
        (self.trial / "events.jsonl").write_text(json.dumps({"operation": "hive", "arguments": ["--resume"],
            "exit_code": 0, "hive_input_bytes": 42}), encoding="utf-8")
        config = {"setup_scope": "user", "user_before": {}, "expected_apply_count": 0,
            "expected_values": {"/interface_language": "ko", "/usage_guard/enforcement_enabled": False},
            "agent_fields": ["/interface_language"], "progress_relative": "user/progress.yml",
            "operation_id": "fixed", "preserved_files": {"user/README.txt": hashlib.sha256(note).hexdigest()}}
        result = self.recorder.collect(self.trial, config, {"status": "completed"})
        self.assertTrue(all(result["checks"].values()), result)
        config["operation_id"] = "another"
        result = self.recorder.collect(self.trial, config, {"status": "completed"})
        self.assertFalse(result["checks"]["same_operation_complete"])

    def test_transport_logs_empty_input_and_accepts_explicit_json_without_running_hive(self):
        tool = self.trial / "tool.py"
        tool.write_bytes((ROOT / "tests/fixtures/interactive-cli/model-trial-tool.py").read_bytes())
        (self.trial / "config.json").write_text("{}", encoding="utf-8")
        rejected = subprocess.run([sys.executable, str(tool), "write-answers"],
            input=b"", capture_output=True, timeout=10)
        self.assertEqual(rejected.returncode, 2)
        self.assertFalse((self.trial / "answers.json").exists())
        first = json.loads((self.trial / "events.jsonl").read_text().splitlines()[0])
        self.assertEqual(first["exit_code"], 2)
        self.assertEqual(len(first["input_sha256"]), 64)
        self.assertEqual(first["hive_input_bytes"], len(rejected.stdout) + len(rejected.stderr))
        payload = {"schema_version": 1, "kind": "hive.setup-answer-patch", "answers": {"interface_language": "ko"}}
        accepted = subprocess.run([sys.executable, str(tool), "write-answers", json.dumps(payload)],
            input=b"", capture_output=True, timeout=10)
        self.assertEqual(accepted.returncode, 0, accepted.stderr)
        self.assertEqual(json.loads((self.trial / "answers.json").read_text()), payload)
        self.assertEqual(len((self.trial / "events.jsonl").read_text().splitlines()), 2)

    def test_skill_entry_is_fixed_and_rejected_paths_get_their_own_guidance(self):
        import hashlib
        tool = self.trial / "tool.py"
        tool.write_bytes((ROOT / "tests/fixtures/interactive-cli/model-trial-tool.py").read_bytes())
        skill = self.trial / "skill/SKILL.md"
        skill.parent.mkdir()
        content = b"Fixed synthetic setup Skill.\n"
        skill.write_bytes(content)
        (self.trial / "config.json").write_text(json.dumps({
            "input_sha256": {"skill/SKILL.md": hashlib.sha256(content).hexdigest()}}))
        accepted = subprocess.run([sys.executable, str(tool), "read-skill"], capture_output=True, timeout=10)
        self.assertEqual(accepted.returncode, 0, accepted.stderr)
        self.assertEqual(accepted.stdout, content)
        refused = subprocess.run([sys.executable, str(tool), "read-skill", "../config.json"],
                                 capture_output=True, timeout=10)
        self.assertEqual(refused.returncode, 2)
        self.assertIn(b"relative to the fixture skill folder", refused.stderr)
        self.assertNotIn(b"write-answers", refused.stderr)
        events = [json.loads(line) for line in (self.trial / "events.jsonl").read_text().splitlines()]
        self.assertEqual(len(events), 2)
        self.assertEqual(events[0]["path"], "SKILL.md")
        self.assertEqual(events[1]["hive_input_bytes"], len(refused.stderr))

    def test_comparison_keeps_legacy_limits_separate_and_rejects_repeated_failures(self):
        user = self.trial / "user"
        (user / ".hive/config").mkdir(parents=True)
        (user / "README.txt").write_bytes(b"Synthetic user note. Preserve exactly.\n")
        (user / ".hive/config/user-setup.yml").write_text('{"interface_language":"ko"}', encoding="utf-8")
        events = [{"operation": "hive", "arguments": ["setup", "--apply"], "exit_code": 0, "hive_input_bytes": 9000}]
        log = self.trial / "events.jsonl"
        log.write_text(json.dumps(events[0]) + "\n", encoding="utf-8")
        config = {"comparison": True, "comparison_version": "0.13.1", "setup_scope": "user",
                  "user_before": {}, "expected_values": {"/interface_language": "ko"}}
        result = self.recorder.collect(self.trial, config, {"status": "completed"})
        self.assertTrue(all(result["checks"].values()), result)
        self.assertNotIn("bounded_responses", result["checks"])
        config["comparison_version"] = "0.14.0"
        result = self.recorder.collect(self.trial, config, {"status": "completed"})
        self.assertFalse(result["checks"]["bounded_responses"])
        self.assertTrue(result["errors"])
        failure = {"operation": "hive", "arguments": ["setup", "--dry-run"], "exit_code": 2,
                   "hive_input_bytes": 80, "result": {"code": "invalid", "message": "unknown flag"}}
        log.write_text("\n".join(json.dumps(e) for e in [failure, failure, *events]), encoding="utf-8")
        config["comparison_version"] = "0.13.1"
        result = self.recorder.collect(self.trial, config, {"status": "completed"})
        self.assertFalse(result["checks"]["no_repeated_failed_input"])
        self.assertEqual(result["hive_input_bytes"], 9160)

    def test_explicit_section_detail_is_separate_from_bounded_question_and_summary(self):
        log = self.trial / "events.jsonl"
        for flags, bounded in [(["--describe", "--section", "usage"], True),
                               (["--describe", "--question", "usage-enforcement"], False),
                               (["--show", "--summary"], False), (["--apply", "--summary"], False)]:
            log.write_text(json.dumps({"operation": "hive", "arguments": flags,
                "exit_code": 0, "hive_input_bytes": 10063}), encoding="utf-8")
            result = self.recorder.collect(self.trial, {"user_before": {}}, {})
            self.assertEqual(result["checks"]["bounded_responses"], bounded, flags)
            self.assertEqual(result["hive_input_bytes"], 10063)

    def test_failure_correction_uses_input_identity_not_only_error_kind(self):
        log = self.trial / "events.jsonl"
        failure = {"operation": "write-answers", "exit_code": 2,
                   "failure_kind": "fixture-input", "hive_input_bytes": 20, "input_sha256": "first"}
        config = {"comparison": True, "comparison_version": "0.14.0", "user_before": {}}
        for second, valid in [("changed", True), ("first", False)]:
            log.write_text("\n".join(json.dumps(e) for e in [failure, {**failure, "input_sha256": second}]), encoding="utf-8")
            result = self.recorder.collect(self.trial, config, {})
            self.assertEqual(result["checks"]["no_repeated_failed_input"], valid)
            self.assertEqual(result["hive_input_bytes"], 40)

    def test_comparison_transport_refuses_foreign_roots_and_optional_activation(self):
        tool = self.trial / "tool.py"
        tool.write_bytes((ROOT / "tests/fixtures/interactive-cli/model-trial-tool.py").read_bytes())
        (self.trial / "config.json").write_text(json.dumps({"comparison": True, "setup_scope": "user",
            "readable_paths": ["user/.hive/config/user-setup.yml"]}), encoding="utf-8")
        commands = [
            ["hive", "setup", "--scope", "user", "--user-root", str(self.trial.parent / "foreign"), "--show"],
            ["hive", "setup", "feature", "answer", "--id", "vector-search", "--answer", "yes", "--user-root", str(self.trial / "user")],
            ["read-input", "../config.json"],
        ]
        for args in commands:
            result = subprocess.run([sys.executable, str(tool), *args], capture_output=True, timeout=10)
            self.assertEqual(result.returncode, 2, result.stdout)
            if "--show" in args:
                self.assertIn(b"expected exactly", result.stderr)
        events = [json.loads(line) for line in (self.trial / "events.jsonl").read_text().splitlines()]
        self.assertEqual(len(events), 3)
        self.assertTrue(all(e["failure_kind"] == "fixture-input" for e in events))
        self.assertEqual(events[0]["arguments"], commands[0][1:])

    def test_comparison_argument_contract_keeps_readonly_help_and_exact_user_scope(self):
        import ast
        source = ROOT / "tests/fixtures/interactive-cli/model-trial-tool.py"
        tree = ast.parse(source.read_text(encoding="utf-8"))
        function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "comparison_arguments")
        values = {"CONFIG": {"comparison": True, "setup_scope": "user"}, "ROOT": self.trial, "Path": Path}
        exec(compile(ast.Module(body=[function], type_ignores=[]), str(source), "exec"), values)
        validate = values["comparison_arguments"]
        for args in [["--version"], ["setup", "--help"], ["setup", "--scope", "user", "--describe", "--output", "json"],
                     ["setup", "--progress", "status", "--scope", "user", "--user-root", str(self.trial / "user"), "--output", "json"]]:
            validate(args)
        for args in [["setup", "--help", "--apply"], ["setup", "--scope", "user", "--user-root", str(self.trial / "user"), "--apply", "--confirm-session-disable"],
                     ["setup", "--scope", "user", "--user-root", str(self.trial.parent), "--show"]]:
            with self.assertRaises((AssertionError, ValueError)):
                validate(args)


if __name__ == "__main__":
    unittest.main()
