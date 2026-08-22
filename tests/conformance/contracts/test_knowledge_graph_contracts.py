"""Derived Markdown and code knowledge graph contract."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[3]
SCHEMA = json.loads((ROOT / "schemas/knowledge-graph.schema.json").read_text(encoding="utf-8"))
DIGEST = "sha256:" + "a1" * 32


class KnowledgeGraphContractTests(unittest.TestCase):
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
