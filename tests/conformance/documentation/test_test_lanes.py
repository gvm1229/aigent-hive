from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts" / "test-lanes.py"
SPEC = importlib.util.spec_from_file_location("test_lanes", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class TestLaneInventoryTests(unittest.TestCase):
    def test_manifest_assigns_each_conformance_module_once(self) -> None:
        lanes = MODULE.load_manifest()
        MODULE.validate_manifest(lanes)
        claimed = [
            module
            for lane in lanes.values()
            for module in lane["modules"]
        ]
        self.assertEqual(set(claimed), MODULE.expected_modules())
        self.assertEqual(len(claimed), len(set(claimed)))

    def test_each_lane_declares_release_metadata(self) -> None:
        for lane in MODULE.load_manifest().values():
            self.assertIsInstance(lane["owner"], str)
            self.assertTrue(lane["contract"])
            self.assertIsInstance(lane["release_gate"], bool)

    def test_changed_path_selection_is_purpose_based_and_unknown_is_full(self) -> None:
        names = list(MODULE.load_manifest())
        self.assertEqual(MODULE.lanes_for_paths(["docs/guide.md"], names), ["documentation"])
        self.assertEqual(
            MODULE.lanes_for_paths(["crates/hive-wiki/src/lib.rs"], names),
            ["security", "contract", "integration"],
        )
        self.assertEqual(MODULE.lanes_for_paths(["unknown.bin"], names), names)

    def test_nested_test_inventory_uses_full_module_path(self) -> None:
        modules = MODULE.expected_modules()
        self.assertIn(
            "tests.conformance.documentation.test_test_lanes",
            modules,
        )
        self.assertNotIn("tests.conformance.test_test_lanes", modules)


if __name__ == "__main__":
    unittest.main()
