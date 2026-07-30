"""Update discovery black-box conformance."""

from __future__ import annotations

import json
import subprocess

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.phase1_support import (
    ACTION_RESULT_SCHEMA,
    Phase1CliTestCase,
    REPOSITORY_ROOT,
)


class UpdateDiscoveryConformance(Phase1CliTestCase):
    def test_default_disabled_check_is_read_only_and_schema_valid(self) -> None:
        before = {
            path.relative_to(self.setup_user_root).as_posix(): path.read_bytes()
            for path in self.setup_user_root.rglob("*")
            if path.is_file()
        }
        process = subprocess.run(
            [
                str(self.hive_binary),
                "update",
                "--check",
                "--user-root",
                str(self.setup_user_root),
                "--output",
                "json",
            ],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        result = json.loads(process.stdout)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        after = {
            path.relative_to(self.setup_user_root).as_posix(): path.read_bytes()
            for path in self.setup_user_root.rglob("*")
            if path.is_file()
        }

        self.assertEqual(process.returncode, 0)
        self.assertEqual(result["code"], "hive.update-check-disabled")
        self.assertEqual(result["data"]["installed"], False)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(before, after)


if __name__ == "__main__":
    import unittest

    unittest.main()
