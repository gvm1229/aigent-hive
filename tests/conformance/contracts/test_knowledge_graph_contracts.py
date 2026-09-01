"""Derived Markdown and code knowledge graph contract."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[3]
SCHEMA = json.loads((ROOT / "schemas/knowledge-graph.schema.json").read_text(encoding="utf-8"))
DIGEST = "sha256:" + "a1" * 32


class KnowledgeGraphContractTests(unittest.TestCase):
    def test_source_graph_qualifies_thirty_exact_and_relation_questions(self) -> None:
        configured = os.environ.get("HIVE_BIN")
        binary = (
            Path(configured).resolve()
            if configured
            else (ROOT / "target/debug/hive").resolve()
        )
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "qualification.json"
            subprocess.run(
                [
                    "python",
                    str(ROOT / "scripts/qualify-source-graph.py"),
                    "--hive",
                    str(binary),
                    "--target",
                    str(ROOT),
                    "--output",
                    str(output),
                ],
                cwd=ROOT,
                check=True,
                timeout=60,
                capture_output=True,
                text=True,
            )
            report = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(report["exact_recall_at_10"], 1.0)
        self.assertEqual(report["relation_grounded_recall_at_10"], 1.0)
        self.assertFalse(report["canonical_changed"])
        self.assertFalse(report["current_canonical_changed"])
        self.assertFalse(report["question_corpus_changed"])
        self.assertEqual(report["facts_revision"], "622f2b7b5411d054abd94c5443ce2b620231b240")
        self.assertEqual(report["question_corpus_digest"], "sha256:" + hashlib.sha256(
            (ROOT / "tests/fixtures/knowledge/vector-gold-120.json").read_bytes()
        ).hexdigest())
        self.assertEqual(report["canonical_tree_digest_before"], report["canonical_tree_digest_after"])
        for field in ("current_source_lint", "frozen_source_lint"):
            self.assertEqual(report[field]["data"]["error_count"], 0)
            self.assertEqual(report[field]["data"]["warning_count"], 0)
        self.assertEqual(report["scope"], "source")

    def test_graphify_three_platform_wheel_locks_are_complete_and_digest_bound(self) -> None:
        root = ROOT / "harness/dependencies/graphify/0.9.47"
        for platform in ("windows-x64", "macos-arm64", "linux-musl-x64"):
            with self.subTest(platform=platform):
                lock = json.loads((root / f"{platform}.json").read_text(encoding="utf-8"))
                self.assertEqual(lock["schema_version"], 1)
                self.assertEqual(lock["package"], "graphifyy==0.9.47")
                self.assertEqual(lock["platform"], platform)
                self.assertEqual(lock["python"], "3.12")
                self.assertEqual(len(lock["files"]), 30)
                self.assertEqual(
                    len({entry["filename"] for entry in lock["files"]}),
                    30,
                )
                self.assertTrue(
                    all(
                        len(entry["sha256"]) == 64
                        and entry["size"] > 0
                        and entry["filename"].endswith(".whl")
                        for entry in lock["files"]
                    )
                )

    def test_native_markdown_generation_is_scope_and_evidence_bound(self) -> None:
        validator = Draft202012Validator(SCHEMA)
        generation = {
            "schema_version": 1,
            "scope": "project",
            "engine": "native-markdown",
            "generation_digest": DIGEST,
            "nodes": [
                {"id": "a", "locator": "Wiki/a.md", "content_digest": DIGEST, "visibility": "project", "lifecycle": "active"},
                {"id": "b", "locator": "Wiki/b.md", "content_digest": DIGEST, "visibility": "project", "lifecycle": "active"},
            ],
            "edges": [{"from": "a", "to": "b", "relation": "links", "evidence": "EXTRACTED", "source_digest": DIGEST}],
        }
        validator.validate(generation)
        generation["edges"][0]["evidence"] = "provider"
        self.assertFalse(validator.is_valid(generation))


if __name__ == "__main__":
    unittest.main()
