"""Development fixture scripts must name the current workspace product version."""

from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[3]
VERSION = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
FIXTURES = [
    "linux-legacy-update.py",
    "linux-monitor-review.py",
    "native-core-removal.py",
    "terminal-setup-ci.py",
    "windows-direct-replace-ci.py",
    "windows-monitor-ci.py",
]


class FixtureProductVersionContract(unittest.TestCase):
    def test_development_fixtures_follow_the_workspace_version(self) -> None:
        major, minor, patch = (int(part) for part in VERSION.split("."))
        stale = {f"{major}.{minor}.{patch - 1}"} if patch else set()
        for name in FIXTURES:
            text = (ROOT / "tests/fixtures/interactive-cli" / name).read_text(encoding="utf-8")
            self.assertIn(VERSION, text, name)
            for old in stale:
                self.assertIsNone(re.search(re.escape(old) + r"(?![0-9])", text), f"{name} still pins {old}")


if __name__ == "__main__":
    unittest.main()
