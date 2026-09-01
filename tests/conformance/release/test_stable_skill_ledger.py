"""Stable Skill ledger publication gate tests."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/check-stable-skill-ledger.py"
SPEC = importlib.util.spec_from_file_location("stable_skill_ledger", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class StableSkillLedgerContract(unittest.TestCase):
    def setUp(self) -> None:
        self.ledger = ROOT / "harness/release/stable-skill-ledger.yml"
        self.historical = ROOT / "harness/skills/historical-builtins.yml"
        self.npm = ["0.8.0", "0.9.0", "0.9.1", "0.9.2", "0.9.3", "0.9.4", "0.9.5", "0.9.5-test.1"]
        self.github = [
            {"tagName": "v0.9.0", "isPrerelease": False},
            {"tagName": "v0.9.1", "isPrerelease": False},
            {"tagName": "v0.9.2", "isPrerelease": False},
            {"tagName": "v0.9.3", "isPrerelease": False},
            {"tagName": "v0.9.4", "isPrerelease": False},
            {"tagName": "v0.9.5", "isPrerelease": False},
            {"tagName": "v0.9.5-test.1", "isPrerelease": True},
        ]

    def test_public_stable_union_and_target_match_current_ledger(self) -> None:
        result = MODULE.verify(self.ledger, self.historical, "0.10.0", self.npm, self.github)
        self.assertEqual(result["stable_versions"][-1], "0.10.0")

    def test_current_target_entry_is_available_for_stable_publication(self) -> None:
        result = MODULE.verify(self.ledger, self.historical, "0.10.0", self.npm, self.github)
        self.assertEqual(result["stable_versions"][-1], "0.10.0")

    def test_missing_future_target_entry_blocks_stable_publication(self) -> None:
        with self.assertRaisesRegex(ValueError, "differs from published stable union"):
            MODULE.verify(self.ledger, self.historical, "0.11.0", self.npm, self.github)

    def test_no_change_epoch_must_have_identical_skill_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory) / "ledger.yml"
            text = self.ledger.read_text(encoding="utf-8").replace(
                "version: 0.9.5, compatibility_epoch: 0.9.4",
                "version: 0.9.5, compatibility_epoch: 0.9.3",
            )
            ledger.write_text(text, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "different Skill bytes"):
                MODULE.verify(ledger, self.historical, "0.9.5", self.npm, self.github)


if __name__ == "__main__":
    unittest.main()
