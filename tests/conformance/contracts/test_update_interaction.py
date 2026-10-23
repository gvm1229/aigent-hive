"""Update input contracts reject missing consent before any installation work."""
from __future__ import annotations

import json
import os
import subprocess

from jsonschema import Draft202012Validator
from tests.conformance.support.harness import ACTION_RESULT_SCHEMA, Phase1CliTestCase


class UpdateInteractionConformance(Phase1CliTestCase):
    def invoke(self, *flags: str, machine: bool = True) -> tuple[subprocess.CompletedProcess, dict | None]:
        root = self.work_root / "untouched-user"
        root.mkdir(exist_ok=True)
        process = subprocess.run(
            [str(self.hive_binary), "update", *flags, "--user-root", str(root),
             *(["--output", "json"] if machine else [])],
            input="yes\nyes\nyes\n", capture_output=True, text=True, encoding="utf-8",
            timeout=10, env={**os.environ, "PATH": str(self.work_root / "no-external-tools")},
        )
        self.assertEqual(list(root.iterdir()), [])
        if machine:
            result = json.loads(process.stdout)
            Draft202012Validator(ACTION_RESULT_SCHEMA).validate(result)
            self.assertEqual(process.returncode, result["exit_code"])
            self.assertEqual(result["changed_paths"], [])
            return process, result
        self.assertEqual(process.stdout, "")
        return process, None

    def test_json_and_noninteractive_do_not_take_pipe_input_as_update_consent(self) -> None:
        for flags in [(), ("--non-interactive",)]:
            with self.subTest(flags=flags):
                _, result = self.invoke(*flags)
                self.assertEqual(result["code"], "hive.update-needs-input")
                self.assertEqual(result["status"], "needs-input")

    def test_explicit_terminal_without_a_terminal_never_applies(self) -> None:
        process, _ = self.invoke("--confirm", "--interactive", machine=False)
        self.assertEqual(process.returncode, 2)
        self.assertIn("usable terminal", process.stderr)

    def test_conflicting_and_duplicate_options_fail_before_mutation(self) -> None:
        for flags in [
            ("--interactive",),
            ("--confirm", "--interactive", "--non-interactive"),
            ("--non-interactive", "--non-interactive"),
            ("--output", "json"),
        ]:
            with self.subTest(flags=flags):
                _, result = self.invoke(*flags)
                self.assertEqual(result["status"], "error")
                self.assertEqual(result["exit_code"], 2)

    def test_input_redirection_alone_does_not_confirm_an_update(self) -> None:
        process, _ = self.invoke(machine=False)
        self.assertEqual(process.returncode, 2)
        self.assertIn("approval is required", process.stderr)
