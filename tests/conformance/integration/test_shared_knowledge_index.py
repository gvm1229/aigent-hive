"""User-root shared knowledge index conformance."""

from __future__ import annotations

import json
import os
import sqlite3
import subprocess
import tempfile
import unittest
from contextlib import closing
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker


ROOT = Path(__file__).resolve().parents[3]
ACTION_SCHEMA = json.loads(
    (ROOT / "schemas/action-result.schema.json").read_text(encoding="utf-8")
)
REGISTRY_SCHEMA = json.loads(
    (ROOT / "schemas/project-registry.schema.json").read_text(encoding="utf-8")
)
QUERY_SCHEMA = json.loads(
    (ROOT / "schemas/knowledge-query-result.schema.json").read_text(encoding="utf-8")
)


def wiki_page(page_id: str, body: str) -> str:
    return f"""\
---
schema_version: 1
id: {page_id}
kind: concept
summary: {page_id} summary
tags: [shared-test]
aliases: []
sources: ["raw:.hive/knowledge/Raw/source-id/{'0' * 64}.md#sha256:{'0' * 64}"]
links: []
contradictions: []
status: active
created_at: 2026-07-28T00:00:00Z
updated_at: 2026-07-28T00:00:00Z
---

{body}
"""


class SharedKnowledgeIndexConformance(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        configured = os.environ.get("HIVE_BIN")
        if configured:
            cls.hive = Path(configured).resolve()
        else:
            subprocess.run(
                ["cargo", "build", "--quiet", "--bin", "hive"],
                cwd=ROOT,
                check=True,
            )
            cls.hive = ROOT / "target/debug" / ("hive.exe" if os.name == "nt" else "hive")

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="aigent-hive-shared-index-")
        root = Path(self.temporary.name).resolve()
        self.user_root = root / "user"
        self.private_project = root / "private-project"
        self.shared_project = root / "shared-project"
        for path in (self.user_root, self.private_project, self.shared_project):
            (path / ".hive/knowledge/Wiki").mkdir(parents=True)
        (self.user_root / ".hive/config").mkdir(parents=True)
        (self.user_root / ".hive/config/user-setup.yml").write_text(
            """\
schema_version: 1
interface_language: en
wiki:
  enabled: true
  language: both
profile:
  id: web-developer
persona:
  id: balanced
selected_hosts:
  - codex
skills:
  mode: individual
  selected:
    - setup-hive
usage_guard:
  enabled: false
  stop_remaining_percent: 20
  codexbar_fallback_enabled: false
""",
            encoding="utf-8",
        )
        self.write_page(self.user_root, "root-page", "root searchable")
        self.write_page(self.private_project, "private-page", "private searchable")
        self.write_page(self.shared_project, "shared-page", "shared searchable")
        self.registry = {
            "schema_version": 1,
            "projects": [
                {
                    "id": "private-project",
                    "root": str(self.private_project),
                    "enabled": True,
                    "language": "en",
                    "visibility": "confidential",
                },
                {
                    "id": "shared-project",
                    "root": str(self.shared_project),
                    "enabled": True,
                    "language": "ko",
                    "visibility": "shared",
                },
            ],
        }
        Draft202012Validator(
            REGISTRY_SCHEMA, format_checker=FormatChecker()
        ).validate(self.registry)
        (self.user_root / ".hive/config/projects.yml").write_text(
            yaml.safe_dump(self.registry, sort_keys=False),
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    @staticmethod
    def write_page(root: Path, page_id: str, body: str) -> None:
        (root / f".hive/knowledge/Wiki/{page_id}.md").write_text(
            wiki_page(page_id, body),
            encoding="utf-8",
        )

    def invoke(self, *arguments: str) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        process = subprocess.run(
            [str(self.hive), *arguments, "--output", "json"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        result = json.loads(process.stdout)
        Draft202012Validator(
            ACTION_SCHEMA, format_checker=FormatChecker()
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        return process, result

    def query(self, target: Path) -> dict[str, object]:
        result = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(target),
            "--user-root",
            str(self.user_root),
            "--text",
            "searchable",
        )[1]
        Draft202012Validator(QUERY_SCHEMA).validate(result["data"])
        return result

    def test_rebuild_uses_only_user_root_database_with_provenance(self) -> None:
        _, result = self.invoke(
            "index",
            "rebuild",
            "--user-root",
            str(self.user_root),
        )
        self.assertEqual(result["status"], "success", result)
        database_path = self.user_root / ".hive/index/hive.sqlite3"
        self.assertTrue(database_path.is_file())
        self.assertFalse((self.private_project / ".hive/index/hive.sqlite3").exists())
        self.assertFalse((self.shared_project / ".hive/index/hive.sqlite3").exists())
        with closing(sqlite3.connect(database_path)) as database:
            row = database.execute(
                """
                SELECT c.source_project_id, d.locator, d.language, d.digest, d.visibility
                FROM documents d
                JOIN collections c ON c.collection_id = d.collection_id
                WHERE d.locator = '.hive/knowledge/Wiki/private-page.md'
                """
            ).fetchone()
        self.assertEqual(
            row[:3],
            ("private-project", ".hive/knowledge/Wiki/private-page.md", "und"),
        )
        self.assertRegex(row[3], r"^sha256:[0-9a-f]{64}$")
        self.assertEqual(row[4], "confidential")

    def test_queries_enforce_current_project_visibility(self) -> None:
        self.invoke("index", "rebuild", "--user-root", str(self.user_root))
        global_ids = [
            hit["page_id"] for hit in self.query(self.user_root)["data"]["hits"]
        ]
        private_ids = [
            hit["page_id"]
            for hit in self.query(self.private_project)["data"]["hits"]
        ]
        shared_ids = [
            hit["page_id"] for hit in self.query(self.shared_project)["data"]["hits"]
        ]
        self.assertEqual(global_ids, ["root-page", "shared-page"])
        self.assertIn("private-page", private_ids)
        self.assertNotIn("private-page", shared_ids)

    def test_canonical_change_requires_deterministic_rebuild(self) -> None:
        self.invoke("index", "rebuild", "--user-root", str(self.user_root))
        self.write_page(self.shared_project, "shared-page", "changed searchable")
        process, result = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(self.user_root),
            "--user-root",
            str(self.user_root),
            "--text",
            "changed",
        )
        self.assertEqual(process.returncode, 0)
        self.assertEqual(result["data"]["hits"], [])
        self.invoke("index", "rebuild", "--user-root", str(self.user_root))
        self.assertEqual(
            [
                hit["page_id"]
                for hit in self.invoke(
                    "knowledge",
                    "query",
                    "--target",
                    str(self.user_root),
                    "--user-root",
                    str(self.user_root),
                    "--text",
                    "changed",
                )[1]["data"]["hits"]
            ],
            ["shared-page"],
        )


if __name__ == "__main__":
    unittest.main()
