from __future__ import annotations

import json
import runpy
import subprocess
import tempfile
import unittest
from pathlib import Path

SOURCE_ROOT = Path(__file__).resolve().parents[3]
SCRIPT = SOURCE_ROOT / "scripts/check-test-release-gate.py"


class AutomaticTestReleaseGate(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        subprocess.run(["git", "config", "user.email", "test@example.invalid"], cwd=self.root, check=True)
        subprocess.run(["git", "config", "user.name", "Test"], cwd=self.root, check=True)
        (self.root / "crates/app").mkdir(parents=True)
        (self.root / "docs/plans/active").mkdir(parents=True)
        (self.root / "crates/app/main.rs").write_text("fn main() {}\n", encoding="utf-8")
        (self.root / "docs/plans/active/work.md").write_text("- [x] [APP10-001] done\n", encoding="utf-8")
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)
        subprocess.run(["git", "commit", "-qm", "base"], cwd=self.root, check=True)
        self.module = runpy.run_path(str(SCRIPT))
        self.module["git"].__globals__["ROOT"] = self.root
        self.base = subprocess.run(["git", "rev-parse", "HEAD"], cwd=self.root, check=True, capture_output=True, text=True).stdout.strip()
        registry = self.root / "docs/public-test-product.json"
        registry.write_text(json.dumps({
            "schema_version": 1,
            "product_version": "0.10.0",
            "accepted_package_version": "0.10.0-test.11",
            "accepted_source_commit": self.base,
            "product_tree_sha256": self.module["product_digest"](self.base),
        }), encoding="utf-8")
        self.module["read_registry"].__globals__["REGISTRY"] = registry
        self.intent = self.root / "docs/test-release-intent.json"
        self.module["read_intent"].__globals__["INTENT"] = self.intent

    def commit(self, path: str, text: str) -> None:
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)
        subprocess.run(["git", "commit", "-qm", path], cwd=self.root, check=True)

    def verify(self, plan_ids: str = "APP10-001") -> dict[str, object]:
        return self.module["verify"]("0.10.0", "0.10.0-test.12", plan_ids, "HEAD")

    def test_completed_product_change_authorizes_automatic_test(self) -> None:
        self.commit("crates/app/main.rs", "fn main() { println!(\"new\"); }\n")
        result = self.verify()
        self.assertEqual(result["status"], "authorized")
        self.assertEqual(result["product_paths"], ["crates/app/main.rs"])

    def test_source_only_changes_never_create_a_numbered_test(self) -> None:
        for path in ("docs/note.md", ".agents/skills/example/SKILL.md", "tests/test_x.py", ".github/workflows/ci.yml"):
            with self.subTest(path=path):
                self.commit(path, "source-only\n")
                with self.assertRaisesRegex(self.module["GateError"], "shipped product bytes did not change"):
                    self.verify()

    def test_missing_or_incomplete_plan_id_refuses_product_change(self) -> None:
        self.commit("crates/app/main.rs", "fn main() { println!(\"new\"); }\n")
        for value in ("", "APP10-002", "REL10-003", "bad"):
            with self.subTest(value=value), self.assertRaises(self.module["GateError"]):
                self.verify(value)

    def test_same_product_digest_refuses_duplicate_number(self) -> None:
        self.commit("docs/note.md", "no product change\n")
        with self.assertRaisesRegex(self.module["GateError"], "shipped product bytes did not change"):
            self.verify()

    def test_registry_digest_tamper_is_refused(self) -> None:
        registry = self.module["read_registry"].__globals__["REGISTRY"]
        value = json.loads(registry.read_text(encoding="utf-8"))
        value["product_tree_sha256"] = "sha256:" + "0" * 64
        registry.write_text(json.dumps(value), encoding="utf-8")
        self.commit("crates/app/main.rs", "fn main() { println!(\"new\"); }\n")
        with self.assertRaisesRegex(self.module["GateError"], "registry digest is stale"):
            self.verify()

    def test_product_path_classifier_excludes_release_scaffolding(self) -> None:
        is_product = self.module["is_product_path"]
        self.assertTrue(is_product("crates/hive-cli/src/main.rs"))
        self.assertTrue(is_product("harness/directives/00-project-harness.md"))
        self.assertFalse(is_product("docs/releases/0.10.0.md"))
        self.assertFalse(is_product(".agents/skills/update-summary/SKILL.md"))
        self.assertFalse(is_product(".github/workflows/release.yml"))

    def test_automatic_intent_supplies_completed_plan_without_user_input(self) -> None:
        self.commit("crates/app/main.rs", "fn main() { println!(\"new\"); }\n")
        self.intent.write_text(json.dumps({
            "schema_version": 1,
            "product_version": "0.10.0",
            "package_version": "0.10.0-test.12",
            "plan_ids": ["APP10-001"],
            "product_tree_sha256": self.module["product_digest"]("HEAD"),
        }), encoding="utf-8")
        self.assertEqual(
            self.module["verify"]("0.10.0", "0.10.0-test.12", None, "HEAD")["status"],
            "authorized",
        )

    def test_automatic_intent_cannot_authorize_another_product_tree(self) -> None:
        self.commit("crates/app/main.rs", "fn main() { println!(\"new\"); }\n")
        self.intent.write_text(json.dumps({
            "schema_version": 1,
            "product_version": "0.10.0",
            "package_version": "0.10.0-test.12",
            "plan_ids": ["APP10-001"],
            "product_tree_sha256": "sha256:" + "0" * 64,
        }), encoding="utf-8")
        with self.assertRaisesRegex(self.module["GateError"], "does not match"):
            self.module["verify"]("0.10.0", "0.10.0-test.12", None, "HEAD")


if __name__ == "__main__":
    unittest.main()
