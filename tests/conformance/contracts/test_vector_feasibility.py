"""Deterministic vector hard-gate corpus and FTS baseline contracts."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CORPUS = ROOT / "tests/fixtures/knowledge/vector-gold-120.json"


class VectorFeasibilityContract(unittest.TestCase):
    def test_corpus_is_reproducible_and_has_exact_category_counts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "corpus.json"
            subprocess.run(
                [
                    sys.executable,
                    str(ROOT / "scripts/build-vector-gold-corpus.py"),
                    "--facts",
                    str(ROOT / "docs/facts"),
                    "--output",
                    str(output),
                ],
                cwd=ROOT,
                check=True,
                timeout=15,
            )
            self.assertEqual(output.read_bytes(), CORPUS.read_bytes())
        corpus = json.loads(CORPUS.read_text(encoding="utf-8"))
        counts = {
            kind: sum(query["kind"] == kind for query in corpus["queries"])
            for kind in ("exact", "paraphrase", "cross-language", "relation")
        }
        self.assertEqual(
            counts,
            {"exact": 30, "paraphrase": 40, "cross-language": 20, "relation": 30},
        )

    def test_fts_baseline_is_reproducible_and_preserves_exact_queries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "baseline.json"
            subprocess.run(
                [
                    sys.executable,
                    str(ROOT / "scripts/benchmark-vector-corpus.py"),
                    "--corpus",
                    str(CORPUS),
                    "--output",
                    str(output),
                    "--repetitions",
                    "1",
                ],
                cwd=ROOT,
                check=True,
                timeout=15,
                capture_output=True,
                text=True,
            )
            baseline = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(baseline["by_kind"]["exact"]["recall_at_10"], 1.0)
        self.assertEqual(baseline["query_count"], 120)
        semantic = (
            baseline["by_kind"]["paraphrase"]["recall_at_10"] * 40
            + baseline["by_kind"]["cross-language"]["recall_at_10"] * 20
        ) / 60
        self.assertLess(semantic, 0.90)


if __name__ == "__main__":
    unittest.main()
