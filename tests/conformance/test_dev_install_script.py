from __future__ import annotations

import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/dev-install.sh"


class DevInstallScriptTests(unittest.TestCase):
    def test_script_has_safe_modes_and_data_boundary(self) -> None:
        text = SCRIPT.read_text(encoding="utf-8")
        for required in (
            "--sandbox",
            "--global",
            "--rollback",
            "$product_version-dev",
            "local developer build",
            "canonical user data unchanged",
            "~/.hive/config",
            "~/.hive/knowledge",
            "project .hive directories",
            "use --rollback first",
            "refusing rollback",
        ):
            self.assertIn(required, text)

    def test_script_parses_with_posix_shell(self) -> None:
        subprocess.run(["sh", "-n", str(SCRIPT)], check=True)
