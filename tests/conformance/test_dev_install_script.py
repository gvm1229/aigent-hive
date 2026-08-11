from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
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
        shell = shutil.which("sh")
        if shell is None:
            self.skipTest("POSIX shell parser is unavailable on this host")
        subprocess.run([shell, "-n", str(SCRIPT)], check=True)

    def test_global_activation_and_rollback_preserve_user_data(self) -> None:
        if os.name == "nt":
            self.skipTest("POSIX developer-install lifecycle is unavailable on Windows")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            home = root / "home"
            bin_dir = root / "bin"
            state_dir = root / "state"
            hive = bin_dir / "hive"
            developer = root / "developer-hive"
            for path, text in (
                (
                    hive,
                    "#!/bin/sh\nprintf '%s\\n' 'AIgent Hive v0.9.0-test #3 · developer test build'\n",
                ),
                (
                    developer,
                    "#!/bin/sh\nprintf '%s\\n' 'AIgent Hive v0.9.0-dev · local developer build'\n",
                ),
            ):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text, encoding="utf-8")
                path.chmod(0o755)
            protected = home / ".hive" / "knowledge" / "preserved.md"
            protected.parent.mkdir(parents=True)
            protected.write_text("canonical knowledge", encoding="utf-8")
            environment = {
                "AIGENT_HIVE_DEV_BINARY": str(developer),
                "HOME": str(home),
                "PATH": f"{bin_dir}:{Path('/usr/bin')}:{Path('/bin')}",
            }

            activated = subprocess.run(
                [str(SCRIPT), "--global", "--state-dir", str(state_dir)],
                check=True,
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
            )
            self.assertIn("developer build activated", activated.stdout)
            self.assertIn("local developer build", hive.read_text(encoding="utf-8"))
            self.assertTrue((state_dir / "original").is_file())
            self.assertEqual(protected.read_text(encoding="utf-8"), "canonical knowledge")

            rolled_back = subprocess.run(
                [str(SCRIPT), "--rollback", "--state-dir", str(state_dir)],
                check=True,
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
            )
            self.assertIn("restored Hive executable", rolled_back.stdout)
            self.assertIn("developer test build", hive.read_text(encoding="utf-8"))
            self.assertFalse(state_dir.exists())
            self.assertEqual(protected.read_text(encoding="utf-8"), "canonical knowledge")
