"""v0.9 public Wiki CLI lifecycle and portability E2E conformance."""

from __future__ import annotations

import json
import subprocess
import time
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.phase1_support import (
    ACTION_RESULT_SCHEMA,
    Phase1CliTestCase,
    REPOSITORY_ROOT,
    snapshot_tree,
    write_operational_user_setup,
)


PHASE2_FIXTURES = REPOSITORY_ROOT / "tests/fixtures/phase2"
SCAN_RESULT_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/knowledge-scan-result.schema.json").read_text(
        encoding="utf-8"
    )
)
RETRIEVAL_RESULT_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/knowledge-retrieval-result.schema.json").read_text(
        encoding="utf-8"
    )
)
IMPORT_RESULT_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/knowledge-import-result.schema.json").read_text(
        encoding="utf-8"
    )
)


class V09WikiCliE2E(Phase1CliTestCase):
    """Exercise public commands through the compiled binary and disposable roots."""

    def setUp(self) -> None:
        super().setUp()
        self.project = self.work_root / "consumer"
        self.project.mkdir()
        process, result = self.invoke_setup(
            self.project,
            capabilities="capabilities-codex-host-native.json",
        )
        self.assertEqual(process.returncode, 0, result)
        self.assertEqual(result["status"], "success", result)

    def invoke_knowledge(
        self,
        *arguments: str,
        user_root: Path | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        root = user_root or self.setup_user_root
        command = [str(self.hive_binary), "knowledge", *arguments]
        if "--user-root" not in command:
            command.extend(["--user-root", str(root)])
        command.extend(["--output", "json"])
        process = subprocess.run(
            command,
            cwd=REPOSITORY_ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"stdout must be one JSON object: {error}\n"
                f"command={command!r}\nstdout={process.stdout!r}\n"
                f"stderr={process.stderr!r}"
            )
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        return process, result

    def assert_success(self, result: dict[str, object]) -> dict[str, object]:
        self.assertEqual(result["status"], "success", result)
        data = result.get("data")
        self.assertIsInstance(data, dict, result)
        return data

    def add_fixture_page(self, name: str, *, quick: bool = False) -> None:
        arguments = [
            "add",
            "--target",
            str(self.project),
            "--source",
            str(PHASE2_FIXTURES / f"raw/{name}.md"),
            "--wiki",
            str(PHASE2_FIXTURES / f"wiki/{name}.md"),
        ]
        if quick:
            arguments.append("--quick")
        data = self.assert_success(self.invoke_knowledge(*arguments)[1])
        self.assertEqual(data["quick"], quick)

    def add_orphan_page(self) -> None:
        source = self.work_root / "orphan-source.md"
        source.write_text("A temporary page used by the delete smoke test.\n", encoding="utf-8")
        draft = self.work_root / "orphan-draft.md"
        draft.write_text(
            """---
schema_version: 1
id: orphan
kind: workflow
summary: Temporary deletion workflow
tags: [cleanup, smoke]
aliases: [temporary-orphan]
sources: [raw:self]
links: []
contradictions: []
status: active
created_at: 2026-08-01T00:00:00Z
updated_at: 2026-08-01T00:00:00Z
---

# Orphan

This temporary page has no backlinks and can be deleted explicitly.
""",
            encoding="utf-8",
        )
        data = self.assert_success(
            self.invoke_knowledge(
                "add",
                "--target",
                str(self.project),
                "--source",
                str(source),
                "--wiki",
                str(draft),
                "--quick",
            )[1]
        )
        self.assertTrue(data["quick"])

    def test_public_wiki_page_verbs_follow_one_canonical_lifecycle(self) -> None:
        """Smoke add/query/lint/list/read/delete/refresh without bypassing the CLI."""
        self.add_fixture_page("alpha", quick=True)
        self.add_fixture_page("beta")
        self.add_orphan_page()

        query = self.assert_success(
            self.invoke_knowledge(
                "query",
                "--target",
                str(self.project),
                "--text",
                "serial canonical writer",
                "--tag",
                "knowledge",
                "--category",
                "synthesis",
                "--limit",
                "5",
            )[1]
        )
        self.assertEqual([hit["id"] for hit in query["hits"]], ["beta"])

        read = self.assert_success(
            self.invoke_knowledge(
                "read",
                "--target",
                str(self.project),
                "--page-id",
                "alpha",
            )[1]
        )
        self.assertEqual(read["outgoing_links"], ["beta"])
        self.assertEqual(read["backlinks"], ["beta"])
        self.assertEqual(read["nonreciprocal_links"], [])

        lint = self.assert_success(
            self.invoke_knowledge("lint", "--target", str(self.project))[1]
        )
        self.assertEqual(lint["error_count"], 0)

        refreshed = self.assert_success(self.invoke_knowledge("refresh")[1])
        self.assertGreaterEqual(refreshed["generation"], 1)

        deleted = self.assert_success(
            self.invoke_knowledge(
                "delete",
                "--target",
                str(self.project),
                "--page-id",
                "orphan",
                "--reason",
                "obsolete",
                "--timestamp",
                "2026-08-01T00:01:00Z",
            )[1]
        )
        self.assertEqual(deleted["page_id"], "orphan")
        self.assertFalse((self.project / ".hive/knowledge/Wiki/orphan.md").exists())

        listed = self.assert_success(
            self.invoke_knowledge(
                "list",
                "--target",
                str(self.project),
                "--category",
                "concept",
            )[1]
        )
        self.assertEqual([page["id"] for page in listed["pages"]], ["alpha"])

    def git(self, target: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", *arguments],
            cwd=target,
            check=True,
            capture_output=True,
            text=True,
        )

    def build_hostile_scan_target(self) -> tuple[Path, bool]:
        target = self.work_root / "hostile-scan"
        target.mkdir()
        self.git(target, "init", "-q")
        (target / "docs").mkdir()
        (target / "vendor").mkdir()
        (target / ".hive").mkdir()
        (target / ".gitignore").write_text(
            "ignored-note.md\n.ignored/\n", encoding="utf-8"
        )
        (target / "README.md").write_text(
            "The fixture documents cobalt retrieval anchors for bounded recall.\n",
            encoding="utf-8",
        )
        (target / "docs/decision.md").write_text(
            "# Decision\n\nOnly agent-reviewed claims may be applied.\n",
            encoding="utf-8",
        )
        (target / "vendor/lib.rs").write_text("foreign vendor bytes\n", encoding="utf-8")
        (target / ".hive/private.md").write_text("runtime state\n", encoding="utf-8")
        (target / ".env").write_text(
            "API_TOKEN=synthetic-hostile-secret\n", encoding="utf-8"
        )
        (target / "binary.md").write_bytes(b"\xff\x00hostile")
        (target / "cache.sqlite3").write_bytes(b"SQLite format 3\x00")
        (target / "notes.md").write_text("Optional untracked project notes.\n", encoding="utf-8")
        (target / "ignored-note.md").write_text(
            "ignored content must not enter inventory\n", encoding="utf-8"
        )
        self.git(
            target,
            "add",
            "--",
            ".gitignore",
            "README.md",
            "docs/decision.md",
            "vendor/lib.rs",
            "binary.md",
            "cache.sqlite3",
        )
        self.git(target, "add", "-f", "--", ".env", ".hive/private.md")

        outside = self.work_root / "outside.md"
        outside.write_text("external content must never be followed\n", encoding="utf-8")
        link = target / "external-link.md"
        try:
            link.symlink_to(outside)
        except OSError:
            linked = False
        else:
            self.git(target, "add", "--", "external-link.md")
            linked = True
        return target, linked

    def test_hostile_scan_apply_and_cross_machine_bounded_retrieve(self) -> None:
        """Cover scan/export/import and both automatic and explicit retrieval bounds."""
        target, linked = self.build_hostile_scan_target()

        inventory = self.assert_success(
            self.invoke_knowledge("scan", "--target", str(target), "--inventory")[1]
        )
        Draft202012Validator(SCAN_RESULT_SCHEMA).validate(inventory)
        entries = {
            entry["relative_path"]: entry
            for entry in inventory["scan"]["inventory"]["entries"]
        }
        self.assertNotIn("notes.md", entries)
        self.assertNotIn("ignored-note.md", entries)
        self.assertEqual(entries["README.md"]["decision"], "included")
        self.assertEqual(entries["vendor/lib.rs"]["reason"], "generated-vendor-runtime-path")
        self.assertEqual(entries[".hive/private.md"]["reason"], "generated-vendor-runtime-path")
        self.assertEqual(entries[".env"]["reason"], "secret-candidate-path")
        self.assertEqual(entries["binary.md"]["reason"], "binary-or-non-utf8")
        self.assertEqual(entries["cache.sqlite3"]["reason"], "unsupported-file-type")
        if linked:
            self.assertEqual(entries["external-link.md"]["reason"], "symlink")
        encoded_inventory = json.dumps(inventory, sort_keys=True)
        self.assertNotIn("synthetic-hostile-secret", encoded_inventory)
        self.assertNotIn("ignored content must not enter inventory", encoded_inventory)

        before_apply_status = self.git(
            target, "status", "--porcelain=v1", "-z", "--untracked-files=all"
        ).stdout
        included = self.assert_success(
            self.invoke_knowledge(
                "scan",
                "--target",
                str(target),
                "--inventory",
                "--include-untracked",
            )[1]
        )
        Draft202012Validator(SCAN_RESULT_SCHEMA).validate(included)
        included_inventory = included["scan"]["inventory"]
        included_entries = {
            entry["relative_path"]: entry for entry in included_inventory["entries"]
        }
        self.assertEqual(included_entries["notes.md"]["decision"], "included")
        self.assertNotIn("ignored-note.md", included_entries)

        review = self.work_root / "reviewed-claims.json"
        review.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "inventory_digest": included_inventory["inventory_digest"],
                    "claims": [
                        {
                            "schema_version": 1,
                            "claim_id": "cobalt-retrieval-anchor",
                            "kind": "project-profile",
                            "statement": "The fixture documents cobalt retrieval anchors for bounded recall.",
                            "version": None,
                            "revision": None,
                            "applicability": None,
                            "evidence": [
                                {
                                    "locator": "README.md",
                                    "content_digest": included_entries["README.md"][
                                        "content_digest"
                                    ],
                                    "kind": "document",
                                }
                            ],
                            "agent_reviewed": True,
                            "global_promotion_candidate": True,
                        }
                    ],
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

        candidates = self.assert_success(
            self.invoke_knowledge(
                "scan",
                "--target",
                str(target),
                "--candidates",
                str(review),
                "--include-untracked",
            )[1]
        )
        Draft202012Validator(SCAN_RESULT_SCHEMA).validate(candidates)
        self.assertFalse(candidates["canonical_mutation"])

        applied = self.assert_success(
            self.invoke_knowledge(
                "scan",
                "--target",
                str(target),
                "--apply",
                str(review),
                "--include-untracked",
            )[1]
        )
        Draft202012Validator(SCAN_RESULT_SCHEMA).validate(applied)
        collection_id = applied["collection"]["collection_id"]
        self.assertRegex(collection_id, r"^collection-[0-9a-f]{64}$")
        self.assertEqual(applied["collection"]["kind"], "directory")
        self.assertFalse(applied["target_mutated"])
        self.assertEqual(
            self.git(target, "status", "--porcelain=v1", "-z", "--untracked-files=all").stdout,
            before_apply_status,
        )

        self.assert_success(self.invoke_knowledge("refresh")[1])
        automatic = self.assert_success(
            self.invoke_knowledge(
                "retrieve",
                "--target",
                str(target),
                "--query",
                "cobalt retrieval anchors",
            )[1]
        )
        Draft202012Validator(RETRIEVAL_RESULT_SCHEMA).validate(automatic)
        self.assertLessEqual(len(automatic["hits"]), 5)
        self.assertLessEqual(automatic["returned_bytes"], 16 * 1024)
        self.assertEqual(automatic["hits"][0]["collection_id"], collection_id)

        bounded = self.assert_success(
            self.invoke_knowledge(
                "retrieve",
                "--target",
                str(target),
                "--query",
                "cobalt retrieval anchors",
                "--scope",
                f"collection:{collection_id}",
                "--top-k",
                "1",
                "--byte-budget",
                "256",
            )[1]
        )
        Draft202012Validator(RETRIEVAL_RESULT_SCHEMA).validate(bounded)
        self.assertEqual(len(bounded["hits"]), 1)
        self.assertLessEqual(bounded["returned_bytes"], 256)
        self.assertEqual(bounded["hits"][0]["item_kind"], "claim")
        self.assertTrue(bounded["hits"][0]["untrusted_content"])

        before_promotion_preview = snapshot_tree(self.setup_user_root)
        promotion_preview = self.assert_success(
            self.invoke_knowledge(
                "promote",
                "--collection",
                collection_id,
                "--review-id",
                "cobalt-retrieval-anchor",
                "--dry-run",
            )[1]
        )
        self.assertEqual(snapshot_tree(self.setup_user_root), before_promotion_preview)
        self.assertFalse(promotion_preview["canonical_mutation"])
        self.assertTrue(promotion_preview["approval_required"])
        promotion_candidate = promotion_preview["candidates"][0]
        source_digest = promotion_candidate["expected_source_digest"]
        self.assertRegex(source_digest, r"^sha256:[0-9a-f]{64}$")
        for field in ("redaction", "deduplication", "contradiction", "replacement"):
            self.assertIn(field, promotion_candidate)

        before_missing_consent = snapshot_tree(self.setup_user_root)
        missing_consent, missing_consent_result = self.invoke_knowledge(
            "promote",
            "--collection",
            collection_id,
            "--review-id",
            "cobalt-retrieval-anchor",
            "--expected-source-digest",
            source_digest,
            "--apply",
        )
        self.assertNotEqual(missing_consent.returncode, 0)
        self.assertEqual(missing_consent_result["status"], "error")
        self.assertEqual(snapshot_tree(self.setup_user_root), before_missing_consent)

        promoted = self.assert_success(
            self.invoke_knowledge(
                "promote",
                "--collection",
                collection_id,
                "--review-id",
                "cobalt-retrieval-anchor",
                "--expected-source-digest",
                source_digest,
                "--confirm-global-promotion",
                "--apply",
            )[1]
        )
        self.assertEqual(promoted["approval"], "explicit-global-promotion")
        self.assertEqual(
            promoted["commit"]["promoted_claim"]["collection_id"], "user-root"
        )

        unrelated = self.work_root / "fresh-unrelated-project"
        unrelated.mkdir()
        setup_process, setup_result = self.invoke_setup(
            unrelated,
            capabilities="capabilities-codex-host-native.json",
        )
        self.assertEqual(setup_process.returncode, 0, setup_result)
        promoted_recall = self.assert_success(
            self.invoke_knowledge(
                "retrieve",
                "--target",
                str(unrelated),
                "--query",
                "cobalt retrieval anchors",
            )[1]
        )
        Draft202012Validator(RETRIEVAL_RESULT_SCHEMA).validate(promoted_recall)
        self.assertTrue(
            any(hit["collection_id"] == "user-root" for hit in promoted_recall["hits"])
        )

        portable_source = self.assert_success(
            self.invoke_knowledge(
                "retrieve",
                "--target",
                str(target),
                "--query",
                "cobalt retrieval anchors",
                "--scope",
                f"collection:{collection_id}",
                "--top-k",
                "1",
                "--byte-budget",
                "256",
            )[1]
        )

        bundle = self.work_root / "scanned-collection.hivekb"
        exported = self.assert_success(
            self.invoke_knowledge(
                "export",
                "--scope",
                f"collection:{collection_id}",
                "--bundle",
                str(bundle),
            )[1]
        )
        self.assertTrue(bundle.is_file())
        self.assertRegex(exported["archive_sha256"], r"^sha256:[0-9a-f]{64}$")

        destination = self.work_root / "fresh-user-root"
        write_operational_user_setup(destination)
        before_dry_run = snapshot_tree(destination)
        dry_run = self.assert_success(
            self.invoke_knowledge(
                "import",
                "--bundle",
                str(bundle),
                "--dry-run",
                user_root=destination,
            )[1]
        )
        Draft202012Validator(IMPORT_RESULT_SCHEMA).validate(dry_run)
        self.assertEqual(dry_run["mode"], "dry-run")
        self.assertEqual(snapshot_tree(destination), before_dry_run)

        imported = self.assert_success(
            self.invoke_knowledge(
                "import",
                "--bundle",
                str(bundle),
                "--apply",
                user_root=destination,
            )[1]
        )
        Draft202012Validator(IMPORT_RESULT_SCHEMA).validate(imported)
        self.assertEqual(imported["mode"], "apply")
        self.assertIn(collection_id, imported["detached_collection_ids"])

        restored_project = self.work_root / "restored-consumer"
        restored_project.mkdir()
        authorization = self.assert_success(
            self.invoke_knowledge(
                "authorize-collection",
                "--operation",
                "attach",
                "--collection",
                collection_id,
                "--target",
                str(restored_project),
                "--expires-at",
                str(int(time.time()) + 30),
                "--nonce",
                "cross-machine-attach-0001",
                "--confirm-current-action",
                user_root=destination,
            )[1]
        )
        attached = self.assert_success(
            self.invoke_knowledge(
                "collection",
                "attach",
                "--collection",
                collection_id,
                "--target",
                str(restored_project),
                "--authorization-id",
                authorization["authorization_id"],
                "--authorization-token",
                authorization["authorization_token"],
                user_root=destination,
            )[1]
        )
        self.assertEqual(attached["collection"]["collection_id"], collection_id)
        self.assertEqual(attached["collection"]["state"], "attached")

        attached_auto = self.assert_success(
            self.invoke_knowledge(
                "retrieve",
                "--target",
                str(restored_project),
                "--query",
                "cobalt retrieval anchors",
                user_root=destination,
            )[1]
        )
        Draft202012Validator(RETRIEVAL_RESULT_SCHEMA).validate(attached_auto)
        self.assertEqual(attached_auto["hits"][0]["collection_id"], collection_id)
        self.assertLessEqual(attached_auto["returned_bytes"], 16 * 1024)

        recalled = self.assert_success(
            self.invoke_knowledge(
                "retrieve",
                "--target",
                str(restored_project),
                "--query",
                "cobalt retrieval anchors",
                "--scope",
                f"collection:{collection_id}",
                "--top-k",
                "1",
                "--byte-budget",
                "256",
                user_root=destination,
            )[1]
        )
        Draft202012Validator(RETRIEVAL_RESULT_SCHEMA).validate(recalled)
        self.assertEqual(
            [hit["digest"] for hit in recalled["hits"]],
            [hit["digest"] for hit in portable_source["hits"]],
        )
        self.assertEqual(
            [hit["locator"] for hit in recalled["hits"]],
            [hit["locator"] for hit in portable_source["hits"]],
        )


if __name__ == "__main__":
    import unittest

    unittest.main()
