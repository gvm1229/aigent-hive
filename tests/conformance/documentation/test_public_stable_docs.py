from __future__ import annotations

import importlib.util
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/check-public-stable-docs.py"
SPEC = importlib.util.spec_from_file_location("public_stable_docs", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class PublicStableDocsTest(unittest.TestCase):
    def copy_fixture(self) -> Path:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        for relative in (
            "README.md",
            "docs/public-stable-release.json",
            "docs/readme/README.ko.md",
            "docs/hive-install-guide.ko.html",
            "docs/overview/product.md",
            "docs/01-index.md",
            "docs/releases/0.10.0.md",
        ):
            source = ROOT / relative
            destination = root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        return root

    def run_check(self, root: Path, *extra: str) -> dict[str, object]:
        command = [sys.executable, str(SCRIPT), "--root", str(root), *extra]
        completed = __import__("subprocess").run(command, capture_output=True, text=True, check=False)
        result = json.loads(completed.stdout)
        result["exit_code"] = completed.returncode
        return result

    def test_current_public_stable_surfaces_pass(self) -> None:
        result = self.run_check(ROOT)
        self.assertEqual(result["exit_code"], 0)
        self.assertEqual(result["status"], "success")

    def test_prerelease_or_stale_marker_fails(self) -> None:
        root = self.copy_fixture()
        readme = root / "README.md"
        readme.write_text(readme.read_text(encoding="utf-8") + "\naigent-hive@test\n", encoding="utf-8")
        result = self.run_check(root)
        self.assertEqual(result["exit_code"], 1)
        self.assertIn("prerelease-exposure", {item["code"] for item in result["failures"]})

        readme.write_text(readme.read_text(encoding="utf-8").replace("version=0.10.0", "version=0.9.5"), encoding="utf-8")
        result = self.run_check(root)
        self.assertIn("stable-marker", {item["code"] for item in result["failures"]})

    def test_coverage_and_channel_boundaries_fail_closed(self) -> None:
        root = self.copy_fixture()
        manifest_path = root / "docs/public-stable-release.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["coverage"].pop("COMPAT-02")
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        result = self.run_check(root)
        self.assertIn("coverage-set", {item["code"] for item in result["failures"]})

        root = self.copy_fixture()
        result = self.run_check(root, "--channel", "stable", "--product-version", "0.10.0", "--release-date", "2026-08-30")
        self.assertIn("stable-target", {item["code"] for item in result["failures"]})

        registry = root / "latest.tsv"
        registry.write_text("aigent-hive\t0.9.5\n@aigent-hive/win32-x64\t0.9.5\n", encoding="utf-8")
        result = self.run_check(root, "--channel", "test", "--registry-latest-file", "latest.tsv")
        self.assertEqual(result["exit_code"], 0)
        registry.write_text("aigent-hive\t0.9.4\n", encoding="utf-8")
        result = self.run_check(root, "--channel", "test", "--registry-latest-file", "latest.tsv")
        self.assertIn("registry-latest", {item["code"] for item in result["failures"]})
