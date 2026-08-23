"""Contracts for the isolated vector requalification pipeline."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/benchmark-vector-requalification.py"
SOURCE = ROOT / "tests/fixtures/knowledge/vector-gold-120.json"


class VectorRequalificationContract(unittest.TestCase):
    def run_script(self, *arguments: str) -> dict[str, object]:
        process = subprocess.run(
            [sys.executable, str(SCRIPT), *arguments],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
            timeout=30,
        )
        return json.loads(process.stdout)

    def test_repeated_and_unique_corpora_are_kept_separate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repeated = self.run_script(
                "generate",
                "--source-corpus",
                str(SOURCE),
                "--output",
                str(root / "repeated.jsonl"),
                "--count",
                "120",
                "--kind",
                "repeated",
            )
            unique = self.run_script(
                "generate",
                "--source-corpus",
                str(SOURCE),
                "--output",
                str(root / "unique.jsonl"),
                "--count",
                "120",
                "--kind",
                "unique",
            )
        self.assertEqual(repeated["unique_text_digests"], 30)
        self.assertEqual(unique["unique_text_digests"], 120)
        self.assertNotEqual(repeated["corpus_digest"], unique["corpus_digest"])

    def test_checkpoint_resume_is_atomic_and_incremental(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            corpus = root / "corpus.jsonl"
            self.run_script(
                "generate",
                "--source-corpus",
                str(SOURCE),
                "--output",
                str(corpus),
                "--count",
                "40",
                "--kind",
                "unique",
            )
            interrupted = self.run_script(
                "build",
                "--corpus",
                str(corpus),
                "--state-dir",
                str(root / "state"),
                "--batch-size",
                "10",
                "--max-batches",
                "1",
            )
            self.assertEqual(interrupted["status"], "interrupted")
            self.assertFalse(interrupted["activated"])
            self.assertFalse((root / "state/current.json").exists())
            completed = self.run_script(
                "build",
                "--corpus",
                str(corpus),
                "--state-dir",
                str(root / "state"),
                "--batch-size",
                "10",
            )
            self.assertEqual(completed["status"], "complete")
            self.assertTrue(completed["activated"])
            first_pointer = completed["pointer_digest"]
            rows = [json.loads(line) for line in corpus.read_text("utf-8").splitlines()]
            rows.pop()
            rows[0]["text"] += " 변경"
            rows.append({"id": "new-record", "scope": "private", "text": "새로운 지식"})
            corpus.write_text(
                "".join(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n" for row in rows),
                encoding="utf-8",
                newline="\n",
            )
            updated = self.run_script(
                "build",
                "--corpus",
                str(corpus),
                "--state-dir",
                str(root / "state"),
                "--batch-size",
                "10",
            )
            self.assertEqual((updated["added"], updated["changed"], updated["deleted"]), (1, 1, 1))
            self.assertNotEqual(updated["pointer_digest"], first_pointer)

    def test_failed_full_build_gate_can_never_adopt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence = {
                "semantic_improvement_points": 15.0,
                "semantic_recall_at_10": 0.933,
                "hybrid_exact_recall_at_10": 1.0,
                "warm_query_p95_ms": 37.4,
                "cold_query_p95_ms": 643.5,
                "full_build_50000_seconds": None,
                "incremental_100_seconds": 8.8,
                "index_bytes": 270_564_639,
                "scope_leaks": 0,
                "network_calls": 0,
                "provider_api_calls": 0,
                "fts_fallback_failures": 0,
                "resume_equivalent": True,
                "accepted_platforms": ["windows-x64"],
            }
            evidence_path = root / "evidence.json"
            evidence_path.write_text(json.dumps(evidence), encoding="utf-8")
            result = self.run_script(
                "decide",
                "--evidence",
                str(evidence_path),
                "--output",
                str(root / "decision.json"),
            )
        self.assertEqual(result["decision"], "defer")
        self.assertFalse(result["product_dependencies_added"])
        self.assertIn("full_build", result["failed_checks"])
        self.assertIn("platforms", result["failed_checks"])

    def test_scope_indexes_use_distinct_physical_roots(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            corpus = root / "corpus.jsonl"
            self.run_script(
                "generate",
                "--source-corpus",
                str(SOURCE),
                "--output",
                str(corpus),
                "--count",
                "60",
                "--kind",
                "unique",
            )
            scope_dirs = set()
            for scope in ("source", "project", "user-root", "shared", "private", "confidential"):
                result = self.run_script(
                    "build",
                    "--corpus",
                    str(corpus),
                    "--state-dir",
                    str(root / "state"),
                    "--scope",
                    scope,
                )
                self.assertEqual(result["entry_count"], 10)
                self.assertEqual(result["scope"], scope)
                scope_dirs.add(result["scope_dir"])
            self.assertEqual(len(scope_dirs), 6)


if __name__ == "__main__":
    unittest.main()
