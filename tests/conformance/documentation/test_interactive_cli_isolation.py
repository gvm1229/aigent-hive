"""No lifecycle mutation or private installation access in the on-host driver."""
from __future__ import annotations

import importlib.util
import os
from pathlib import Path
import subprocess
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


if __name__ == "__main__":
    unittest.main()
