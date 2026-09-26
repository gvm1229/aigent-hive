"""Reading budgets must include applicable procedures and reject unsafe or incomplete inputs."""
import copy
import json
import runpy
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CHECK = runpy.run_path(str(ROOT / "scripts/check-agent-directives.py"))["check_read_budgets"]


class DirectiveReadingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.data = json.loads((ROOT / ".agents/directives/read-budgets.json").read_text(encoding="utf-8"))
        for case in self.data["scenarios"].values():
            for relative in case["files"]:
                path = self.root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("Bounded directive.\n", encoding="utf-8")
        self.save()

    def save(self):
        (self.root / ".agents/directives/read-budgets.json").write_text(json.dumps(self.data), encoding="utf-8")

    def test_includes_reference_bytes_and_detects_regression(self):
        metrics, errors = CHECK(self.root)
        self.assertEqual(errors, [])
        path = self.root / ".agents/directives/references/git-commits.md"
        before = path.stat().st_size
        path.write_bytes(b"x" * 40000)
        after, errors = CHECK(self.root)
        self.assertEqual(after["code-edit"]["read_bytes"] - metrics["code-edit"]["read_bytes"], 40000 - before)
        self.assertTrue(any(e["code"] == "phase-read-budget" for e in errors))

    def test_cannot_hide_mandatory_procedure_or_double_count(self):
        original = copy.deepcopy(self.data)
        for mutation in ("omit", "duplicate"):
            self.data = copy.deepcopy(original)
            files = self.data["scenarios"]["plan-authoring"]["files"]
            if mutation == "omit": files.remove(".agents/directives/references/planning-contract.md")
            else: files.append(files[0])
            self.save()
            self.assertTrue(CHECK(self.root)[1])

    def test_missing_and_escaped_paths_fail(self):
        original = copy.deepcopy(self.data)
        for bad in ("../outside.md", "/outside.md", "C:/outside.md", ".agents/directives/missing.md"):
            self.data = copy.deepcopy(original)
            self.data["scenarios"]["startup"]["files"].append(bad)
            self.save()
            self.assertTrue(CHECK(self.root)[1])

    def test_corrupt_or_oversized_definition_fails(self):
        path = self.root / ".agents/directives/read-budgets.json"
        for content in (b"{", b" " * 65537):
            path.write_bytes(content)
            self.assertTrue(CHECK(self.root)[1])
