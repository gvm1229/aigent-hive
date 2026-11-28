"""Contract for byte-for-byte upstream vendoring and the weekly sync report."""

from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("sync_upstream", ROOT / "scripts/sync-upstream.py")
sync_upstream = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(sync_upstream)


def git(*args: str, cwd: Path) -> str:
    environment = {**os.environ, "GIT_AUTHOR_NAME": "Upstream", "GIT_AUTHOR_EMAIL": "upstream@example.invalid",
                   "GIT_COMMITTER_NAME": "Upstream", "GIT_COMMITTER_EMAIL": "upstream@example.invalid"}
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True,
                          env=environment).stdout.strip()


class SyncUpstreamContract(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.base = Path(self.temporary.name)
        self.remotes = self.base / "remotes"
        self.root = self.base / "hive"
        (self.root / "harness/upstream").mkdir(parents=True)
        self.previous = os.environ.get("HIVE_UPSTREAM_BASE_URL")
        os.environ["HIVE_UPSTREAM_BASE_URL"] = self.remotes.as_uri()

    def tearDown(self) -> None:
        if self.previous is None:
            os.environ.pop("HIVE_UPSTREAM_BASE_URL", None)
        else:
            os.environ["HIVE_UPSTREAM_BASE_URL"] = self.previous
        self.temporary.cleanup()

    def publish(self, repo: str, files: dict[str, bytes]) -> str:
        work = self.base / "work" / repo
        bare = self.remotes / f"{repo}.git"
        if not bare.exists():
            bare.parent.mkdir(parents=True, exist_ok=True)
            git("init", "--quiet", "--bare", "--initial-branch=main", str(bare), cwd=self.base)
            work.mkdir(parents=True)
            git("init", "--quiet", "--initial-branch=main", cwd=work)
            git("remote", "add", "origin", str(bare), cwd=work)
        for relative, payload in files.items():
            target = work / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(payload)
        git("add", "-A", cwd=work)
        git("commit", "--quiet", "-m", "update", cwd=work)
        git("push", "--quiet", "origin", "main", cwd=work)
        return git("rev-parse", "HEAD", cwd=work)

    def write_lock(self, *sources: dict) -> None:
        lock = {"schema_version": 1, "sources": list(sources)}
        (self.root / "harness/upstream/UPSTREAM.lock").write_text(json.dumps(lock), encoding="utf-8")

    def source(self, name: str, repo: str, commit: str = "") -> dict:
        return {"name": name, "repo": repo, "ref": "main", "path": "skills/demo", "commit": commit,
                "license": "MIT", "license_files": ["LICENSE"], "used_by": ["design-curate"], "synced_at": ""}

    def test_sync_copies_exact_bytes_then_reports_unchanged_changed_and_unreachable(self) -> None:
        crlf = b"@echo off\r\nexit /b 0\r\n"
        first = self.publish("acme/demo", {"LICENSE": b"MIT License\n", "skills/demo/SKILL.md": b"# Demo\n",
                                           "skills/demo/run.cmd": crlf, "README.md": b"outside\n"})
        self.write_lock(self.source("demo", "acme/demo"), self.source("gone", "acme/gone"))

        report = sync_upstream.sync(self.root, check=False, today="2026-11-30")
        statuses = {entry["name"]: entry for entry in report["sources"]}
        self.assertEqual(statuses["demo"]["status"], "changed")
        self.assertEqual(statuses["gone"]["status"], "skipped")
        self.assertIn("reason", statuses["gone"])
        vendored = self.root / "harness/upstream/demo"
        self.assertEqual((vendored / "run.cmd").read_bytes(), crlf)
        self.assertEqual((vendored / "LICENSE").read_bytes(), b"MIT License\n")
        self.assertFalse((vendored / "README.md").exists())
        lock = json.loads((self.root / "harness/upstream/UPSTREAM.lock").read_text(encoding="utf-8"))
        self.assertEqual(lock["sources"][0]["commit"], first)
        self.assertEqual(lock["sources"][0]["synced_at"], "2026-11-30")
        self.assertEqual(lock["sources"][1]["commit"], "")

        again = sync_upstream.sync(self.root, check=True)
        self.assertEqual(again["sources"][0]["status"], "unchanged")
        self.assertFalse(again["changed"])

        conflicts = self.root / sync_upstream.CONFLICTS
        conflicts.parent.mkdir(parents=True)
        conflicts.write_text("| Topic | Conflicting rules | Sources | Ruling | Reason |\n| --- | --- | --- | --- | --- |\n"
                             "| Motion | a vs b | demo, other | one moment | shared |\n"
                             "| Fonts | c vs d | demolition | x | y |\n", encoding="utf-8")
        second = self.publish("acme/demo", {"skills/demo/SKILL.md": b"# Demo v2\n"})
        checked = sync_upstream.sync(self.root, check=True)
        entry = checked["sources"][0]
        self.assertEqual(entry["status"], "changed")
        self.assertEqual(entry["changes"]["modified"], ["SKILL.md"])
        self.assertEqual(entry["conflict_topics"], ["Motion"])
        self.assertEqual((vendored / "SKILL.md").read_bytes(), b"# Demo\n", "check mode must not write")
        body = sync_upstream.pull_request_body(checked)
        self.assertIn("Revisit conflict ruling: Motion", body)
        self.assertIn("`gone` (acme/gone): skipped", body)

        sync_upstream.sync(self.root, check=False, today="2026-12-07")
        self.assertEqual((vendored / "SKILL.md").read_bytes(), b"# Demo v2\n")
        lock = json.loads((self.root / "harness/upstream/UPSTREAM.lock").read_text(encoding="utf-8"))
        self.assertEqual(lock["sources"][0]["commit"], second)

    def test_lock_rejects_unsafe_paths_and_missing_licenses(self) -> None:
        for bad in ({"path": "../escape"}, {"license_files": []}, {"license_files": ["/abs"]}, {"used_by": []}):
            source = {**self.source("demo", "acme/demo"), **bad}
            self.write_lock(source)
            with self.assertRaises(sync_upstream.SyncError):
                sync_upstream.load_lock(self.root)

    def test_real_conflict_rulings_name_topics_to_revisit_for_each_design_source(self) -> None:
        topics = {name: sync_upstream.conflict_topics(ROOT, name)
                  for name in ("hallmark", "frontend-design", "web-artifacts-builder", "impeccable")}
        self.assertIn("Labels and eyebrows", topics["hallmark"])
        self.assertIn("Density and word count", topics["impeccable"])
        self.assertIn("Typeface choice", topics["web-artifacts-builder"])
        self.assertTrue(all(topics.values()), topics)
        self.assertEqual(sync_upstream.conflict_topics(ROOT, "omc-ai-slop-cleaner"), [])
        report = {"sources": [{"name": "hallmark", "repo": "nutlope/hallmark", "status": "changed",
                               "previous_commit": "a" * 40, "commit": "b" * 40, "used_by": ["design-curate"],
                               "changes": {"added": [], "modified": ["SKILL.md"], "removed": []},
                               "conflict_topics": topics["hallmark"]}]}
        body = sync_upstream.pull_request_body(report)
        self.assertIn("`hallmark` (nutlope/hallmark)", body)
        self.assertIn("Revisit conflict ruling: Labels and eyebrows", body)

    def test_checked_in_vendored_trees_carry_their_licenses(self) -> None:
        self.assertEqual(sync_upstream.verify(ROOT), [])
        lock = sync_upstream.load_lock(ROOT)
        names = {source["name"] for source in lock["sources"]}
        self.assertEqual(names, {"omc-ai-slop-cleaner", "hallmark", "frontend-design",
                                 "web-artifacts-builder", "impeccable"})
        for source in lock["sources"]:
            self.assertRegex(source["commit"], r"^[0-9a-f]{40}$")
        for directory in (ROOT / "harness/upstream").iterdir():
            if directory.is_dir():
                self.assertIn(directory.name, names)


if __name__ == "__main__":
    unittest.main()
