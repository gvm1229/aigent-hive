from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
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
            "global developer replacement is disabled",
            "refusing rollback",
        ):
            self.assertIn(required, text)

    def test_script_parses_with_posix_shell(self) -> None:
        shell = "C:/Program Files/Git/bin/bash.exe" if os.name == "nt" else shutil.which("sh")
        if shell is None:
            self.skipTest("POSIX shell parser is unavailable on this host")
        subprocess.run([shell, "-n", str(SCRIPT)], check=True)

    def test_global_activation_is_refused_before_build_or_mutation(self) -> None:
        shell = ("C:/Program Files/Git/bin/bash.exe" if os.name == "nt" else shutil.which("sh"))
        if not shell or not Path(shell).is_file():
            self.skipTest("POSIX shell is unavailable on this host")
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
                [shell, str(SCRIPT), "--global", "--state-dir", str(state_dir)],
                check=False,
                cwd=ROOT,
                env={**os.environ, **environment},
                capture_output=True,
                text=True,
            )
            self.assertEqual(activated.returncode, 3)
            self.assertIn("global developer replacement is disabled", activated.stderr)
            self.assertIn("developer test build", hive.read_text(encoding="utf-8"))
            self.assertFalse(state_dir.exists())
            self.assertEqual(protected.read_text(encoding="utf-8"), "canonical knowledge")
