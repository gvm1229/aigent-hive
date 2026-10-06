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
from tests.conformance.support.harness import Phase1CliTestCase


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


class HostSemanticGraphCliTests(Phase1CliTestCase):
    def command(self, *args: str, expect: int = 0) -> dict:
        process = subprocess.run([str(self.hive_binary), *args, "--output", "json"],
            capture_output=True, text=True, encoding="utf-8", cwd=ROOT, timeout=30)
        data = json.loads(process.stdout)
        self.assertEqual(process.returncode, expect, data)
        return data

    def remember(self, key: str, fact: str) -> dict:
        return self.command("knowledge", "remember", "--user-root", str(self.setup_user_root),
            "--user-statement", fact, "--claim-key", key, "--kind", "convention")

    def graph(self, action: str, *args: str, expect: int = 0) -> dict:
        return self.command("knowledge", "graph", action, "--engine", "host-semantic",
            "--target", str(self.setup_user_root), "--user-root", str(self.setup_user_root),
            "--collection", "user-root", "--visibility", "shared", "--host", "codex", *args, expect=expect)

    def enable(self) -> None:
        self.remember("semantic-alpha", "Alpha depends on Beta.")
        self.remember("semantic-beta", "Beta is foundational.")
        preview = self.graph("preview")["data"]
        self.graph("enable", "--consent-digest", preview["consent_digest"])

    def result_files(self, batch: dict, *, valid: bool = True) -> tuple[Path, Path]:
        result = {"schema_version": 1, "request_digest": batch["request_digest"],
            "processed_ids": [doc["id"] for doc in batch["documents"]], "relations": []}
        encoded = json.dumps(result, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
        receipt = {"schema_version": 1, "request_digest": batch["request_digest"],
            "result_digest": "sha256:" + hashlib.sha256(encoded).hexdigest(),
            "host": "codex", "reviewed": valid, "attempt": batch["next_attempt"]}
        first, second = self.work_root / "result.json", self.work_root / "receipt.json"
        first.write_bytes(encoded)
        second.write_text(json.dumps(receipt), encoding="utf-8")
        return first, second

    def test_capture_prepare_apply_noop_and_stale_result(self) -> None:
        self.enable()
        batch = self.graph("prepare")["data"]["request"]
        self.assertEqual(len(batch["documents"]), 2)
        first, second = self.result_files(batch)
        self.graph("apply", "--request-digest", batch["request_digest"],
            "--input", str(first), "--receipt", str(second))
        self.assertEqual(self.graph("status")["data"]["pending_count"], 0)
        noop = self.remember("semantic-alpha", "Alpha depends on Beta.")
        self.assertEqual(noop["data"]["graph_update"]["state"], "unchanged")
        queued = self.remember("semantic-gamma", "Gamma uses Alpha.")
        self.assertEqual(queued["data"]["graph_update"]["state"], "pending")
        result = self.graph("apply", "--request-digest", batch["request_digest"],
            "--input", str(first), "--receipt", str(second), expect=2)
        self.assertIn("stale", result["message"])
        self.assertEqual(self.graph("status")["data"]["pending_count"], 1)

    def test_two_failed_reviews_exhaust_correction_budget(self) -> None:
        self.enable()
        request_digest = None
        for attempt in (1, 2):
            batch = self.graph("prepare")["data"]["request"]
            self.assertEqual(batch["next_attempt"], attempt)
            if request_digest is not None:
                self.assertEqual(batch["request_digest"], request_digest)
            request_digest = batch["request_digest"]
            first, second = self.result_files(batch, valid=False)
            self.graph("apply", "--request-digest", request_digest,
                "--input", str(first), "--receipt", str(second), expect=5)
        prepared = self.graph("prepare")["data"]
        self.assertFalse(prepared["analysis_allowed"])
        self.assertFalse(prepared["needs_model"])

    def test_deleted_documents_hide_relations_and_cleanup_without_analysis(self) -> None:
        self.enable()
        batch = self.graph("prepare")["data"]["request"]
        source = next(d for d in batch["documents"] if d["text"].startswith("Alpha"))
        target = next(d for d in batch["documents"] if d["text"].startswith("Beta"))
        first, second = self.result_files(batch)
        result = json.loads(first.read_bytes())
        result["relations"] = [{"from": source["id"], "to": target["id"],
            "kind": "depends-on", "evidence": "INFERRED", "start": 0,
            "end": len(source["text"].encode()),
            "text_digest": "sha256:" + hashlib.sha256(source["text"].encode()).hexdigest(),
            "source_digest": source["digest"], "target_digest": target["digest"]}]
        encoded = json.dumps(result, sort_keys=True, separators=(",", ":")).encode()
        first.write_bytes(encoded)
        receipt = json.loads(second.read_bytes())
        receipt["result_digest"] = "sha256:" + hashlib.sha256(encoded).hexdigest()
        second.write_text(json.dumps(receipt), encoding="utf-8")
        self.graph("apply", "--request-digest", batch["request_digest"],
            "--input", str(first), "--receipt", str(second))
        self.assertEqual(self.graph("status")["data"]["relations"], 1)
        for document in batch["documents"]:
            canonical = self.setup_user_root / document["locator"].split("#", 1)[0]
            self.assertTrue(canonical.resolve().is_relative_to(self.setup_user_root.resolve()))
            canonical.unlink()
        self.command("index", "rebuild", "--user-root", str(self.setup_user_root))
        self.assertEqual(self.graph("status")["data"]["relations"], 0)
        prepared = self.graph("prepare")["data"]
        self.assertFalse(prepared["needs_model"])
        self.assertTrue(prepared["cleanup_required"])
        batch = prepared["request"]
        first, second = self.result_files(batch)
        self.graph("apply", "--request-digest", batch["request_digest"],
            "--input", str(first), "--receipt", str(second))
        self.assertFalse(self.graph("prepare")["data"]["cleanup_required"])

    def test_broken_derived_state_cannot_undo_canonical_capture(self) -> None:
        self.enable()
        state = next((self.setup_user_root / ".hive/index/semantic-graph").glob("*.json"))
        state.write_bytes(b"broken")
        result = self.remember("semantic-delta", "Delta is retained despite graph failure.")
        self.assertEqual(result["data"]["graph_update"]["reason"], "derived-state-unavailable")
        self.assertEqual(result["data"]["plan"]["disposition"], "insert")
        again = self.remember("semantic-delta", "Delta is retained despite graph failure.")
        self.assertEqual(again["data"]["plan"]["disposition"], "noop")

    def test_source_ingest_queues_once_and_graph_failure_preserves_storage(self) -> None:
        self.enable()
        (self.setup_user_root / ".hive/knowledge/suppression.yml").write_bytes(
            (ROOT / "harness/template/.hive/knowledge/suppression.yml").read_bytes())
        batch = self.graph("prepare")["data"]["request"]
        first, second = self.result_files(batch)
        self.graph("apply", "--request-digest", batch["request_digest"],
            "--input", str(first), "--receipt", str(second))
        source = self.work_root / "public-source.md"
        source.write_text("Public source for semantic ingest.\n", encoding="utf-8")
        draft = self.work_root / "draft.md"
        draft.write_text("""---
schema_version: 1
id: semantic-source
kind: concept
summary: Stored source
tags: [semantic]
aliases: []
sources: [raw:self]
links: []
contradictions: []
status: active
created_at: 2026-10-05T00:00:00Z
updated_at: 2026-10-05T00:00:00Z
---

Gamma retains public source evidence.
""", encoding="utf-8")
        args = ["--target", str(self.setup_user_root), "--user-root", str(self.setup_user_root),
            "--source", str(source), "--wiki", str(draft)]
        added = self.command("knowledge", "ingest", *args)
        self.assertEqual(added["data"]["graph_update"]["state"], "pending")
        self.assertTrue(added["data"]["graph_update"]["changed"])
        batch = self.graph("prepare")["data"]["request"]
        self.assertEqual(len(batch["documents"]), 1)
        first, second = self.result_files(batch)
        self.graph("apply", "--request-digest", batch["request_digest"],
            "--input", str(first), "--receipt", str(second))
        repeated = self.command("knowledge", "add", *args)
        self.assertEqual(repeated["data"]["graph_update"]["state"], "unchanged")
        state = next((self.setup_user_root / ".hive/index/semantic-graph").glob("*.json"))
        state.write_bytes(b"broken")
        source.write_text("Revised public source for semantic ingest.\n", encoding="utf-8")
        saved = self.command("knowledge", "ingest", *args)
        self.assertEqual(saved["data"]["graph_update"]["reason"], "derived-state-unavailable")
        self.assertTrue((self.setup_user_root / ".hive/knowledge/Wiki/semantic-source.md").is_file())

    def test_source_partition_keeps_consumer_tree_absent(self) -> None:
        source = self.work_root / "source"
        (source / "docs/facts/en").mkdir(parents=True)
        (source / "docs/facts/ko").mkdir()
        (source / "hive-source.json").write_text(json.dumps({"schema_version": 1,
            "kind": "aigent-hive-source-workspace", "consumer_setup_allowed": False}), encoding="utf-8")
        (source / "docs/source.md").write_bytes(b"canonical\n")
        fingerprint = hashlib.sha256(b"canonical\n").hexdigest()
        for language, other in [("en", "ko"), ("ko", "en")]:
            page = f'''---
schema_version: 1
pair_id: alpha
topic_slug: alpha
language: {language}
counterpart: ../{other}/alpha.md
title: Alpha
summary: Canonical source
tags: [test]
aliases: []
sources: ["repo:docs/source.md#sha256:{fingerprint}"]
links: []
reviewed_revision: "git:{'a' * 40}"
status: active
---

# Alpha

Canonical source.
'''
            (source / f"docs/facts/{language}/alpha.md").write_bytes(page.encode())
        self.command("source-wiki", "index", "--target", str(source))
        args = ["--engine", "host-semantic", "--target", str(source), "--language", "en", "--host", "codex"]
        preview = self.command("source-wiki", "graph", "preview", *args)["data"]
        self.command("source-wiki", "graph", "enable", *args, "--consent-digest", preview["consent_digest"])
        batch = self.command("source-wiki", "graph", "prepare", *args)["data"]["request"]
        first, second = self.result_files(batch)
        self.command("source-wiki", "graph", "apply", *args, "--request-digest", batch["request_digest"],
            "--input", str(first), "--receipt", str(second))
        self.assertFalse((source / ".hive").exists())
        self.assertEqual(self.command("source-wiki", "graph", "status", *args)["data"]["pending_count"], 0)
        self.assertTrue((source / ".agents/work/semantic-graph").is_dir())

    def test_confidential_scope_without_action_grant_never_returns_documents(self) -> None:
        self.enable()
        process = subprocess.run([str(self.hive_binary), "knowledge", "graph", "prepare",
            "--engine", "host-semantic", "--target", str(self.setup_user_root),
            "--user-root", str(self.setup_user_root), "--collection", "user-root",
            "--visibility", "confidential", "--host", "codex", "--output", "json"],
            capture_output=True, text=True, encoding="utf-8", timeout=30)
        self.assertNotEqual(process.returncode, 0)
        self.assertNotIn("Alpha depends", process.stdout)


if __name__ == "__main__":
    unittest.main()
