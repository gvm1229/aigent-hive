"""Update discovery black-box conformance."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.support.harness import (
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

    def test_bare_update_requires_a_terminal_without_user_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            user_root = Path(temporary)
            sentinel = user_root / "sentinel"
            sentinel.write_bytes(b"preserve")
            environment = os.environ.copy()
            environment["USERPROFILE" if os.name == "nt" else "HOME"] = str(user_root)

            process = subprocess.run(
                [str(self.hive_binary), "update"],
                cwd=REPOSITORY_ROOT,
                env=environment,
                check=False,
                text=True,
                input="yes\n",
                capture_output=True,
            )

            self.assertEqual(process.returncode, 3)
            self.assertIn("interactive terminal", process.stderr)
            self.assertEqual(
                {
                    path.relative_to(user_root).as_posix(): path.read_bytes()
                    for path in user_root.rglob("*")
                    if path.is_file()
                },
                {"sentinel": b"preserve"},
            )


if __name__ == "__main__":
    import unittest

    unittest.main()
