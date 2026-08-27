#!/usr/bin/env python3
"""Conformance gates for the v0.9 typed knowledge JSON contracts."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator, FormatChecker


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
SCHEMA_ROOT = REPOSITORY_ROOT / "schemas"
SCHEMA_NAMES = (
    "knowledge-collection-registry.schema.json",
    "knowledge-remember-request.schema.json",
    "knowledge-retrieve-request.schema.json",
    "knowledge-retrieval-result.schema.json",
    "knowledge-bundle-manifest.schema.json",
    "knowledge-import-result.schema.json",
    "knowledge-scan-review.schema.json",
    "knowledge-scan-result.schema.json",
)
COLLECTION_TABLES = (
    "collections",
    "documents",
    "chunks",
    "claims",
    "sources",
    "links",
    "replacements",
)


def digest(character: str = "a") -> str:
    return f"sha256:{character * 64}"


def collection_id(character: str = "a") -> str:
    return f"collection-{character * 64}"


def claim_id(character: str = "c") -> str:
    return f"claim-{character * 64}"


def schema(name: str) -> dict[str, Any]:
    return json.loads((SCHEMA_ROOT / name).read_text(encoding="utf-8"))


def validator(name: str) -> Draft202012Validator:
    return Draft202012Validator(schema(name), format_checker=FormatChecker())


def no_rollback() -> dict[str, Any]:
    return {
        "attempted": False,
        "succeeded": False,
        "backup_digest": None,
        "restored_paths": [],
    }


def collection_registry() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "collections": [
            {
                "collection_id": "user-root",
                "kind": "user-root",
                "state": "attached",
                "aliases": ["Personal"],
                "local_locator": "C:\\Users\\tester\\.hive\\knowledge",
                "default_visibility": "shared",
            },
            {
                "collection_id": collection_id(),
                "kind": "registered-project",
                "state": "detached",
                "aliases": [],
                "source_project_id": "project-alpha",
                "default_visibility": "project-private",
            },
        ],
    }


def remember_request() -> dict[str, Any]:
    return {
        "collection_id": collection_id(),
        "claim_key": "release-policy",
        "claim_id": claim_id(),
        "locator": "docs/facts/release-policy.md",
        "kind": "decision",
        "status": "verified",
        "visibility": "project-private",
        "normalized_fact": "Publishing requires an explicit user request.",
        "provenance": {
            "source_kind": "reviewed-artifact",
            "summary": "Reviewed release policy decision.",
            "locator": "docs/decisions/product-release-decisions.md",
            "digest": digest("b"),
        },
        "sources": ["docs/decisions/product-release-decisions.md"],
        "supersedes": [claim_id("d")],
        "expected_active_digest": digest("e"),
        "observed_at": "2026-08-01T00:00:00Z",
        "verified_at": "2026-08-01T00:01:00Z",
    }


def retrieve_request() -> dict[str, Any]:
    return {
        "scope": {"collection": collection_id()},
        "current_collection_id": collection_id(),
        "query": "release policy",
        "query_expansions": ["publishing approval"],
        "top_k": 10,
        "byte_budget": 65536,
        "confidential_collection_id": collection_id(),
    }


def retrieval_result() -> dict[str, Any]:
    return {
        "generation": 4,
        "manifest_digest": digest("f"),
        "hits": [
            {
                "chunk_id": f"chunk-{digest('1')}",
                "collection_id": collection_id(),
                "item_id": claim_id(),
                "item_kind": "claim",
                "locator": "docs/facts/release-policy.md#chunk=0",
                "title": "Release policy",
                "text": "Publishing requires an explicit user request.",
                "digest": digest("2"),
                "visibility": "confidential",
                "language": "en",
                "claim_kind": "decision",
                "assertion_status": "verified",
                "scan_metadata": {
                    "review_id": "release-policy",
                    "version": "0.9.0",
                    "source_revision": "release-2026-08",
                    "applicability": "Aigent Hive release operations.",
                    "evidence": [
                        {
                            "locator": "docs/decisions/product-release-decisions.md",
                            "content_digest": digest("b"),
                            "kind": "document",
                        }
                    ],
                    "review_status": "agent-reviewed",
                    "global_promotion_candidate": True,
                    "promotion_status": "pending-review",
                },
                "score": 1.5,
                "matched_field": "text",
                "sources": ["docs/decisions/product-release-decisions.md"],
                "untrusted_content": True,
            }
        ],
        "returned_bytes": 45,
        "insufficient_budget": False,
    }


def bundle_source() -> dict[str, Any]:
    return {
        "kind": "project",
        "id": "project-alpha",
        "logical_digest": digest("3"),
    }


def bundle_scope() -> dict[str, Any]:
    return {"kind": "project", "id": "project-alpha"}


def bundle_manifest() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "source": bundle_source(),
        "scope": bundle_scope(),
        "entries": [
            {
                "path": "data/.hive/knowledge/Wiki/release-policy.md",
                "length": 45,
                "sha256": digest("4"),
                "classification": "canonical-markdown",
            }
        ],
    }


def import_result(disposition: str = "planned") -> dict[str, Any]:
    result: dict[str, Any] = {
        "mode": "dry-run",
        "disposition": disposition,
        "archive_sha256": digest("5"),
        "manifest_sha256": digest("6"),
        "source": bundle_source(),
        "scope": bundle_scope(),
        "entry_count": 1,
        "added_count": 1,
        "unchanged_count": 0,
        "detached_collection_ids": [collection_id()],
        "changed_paths": [],
        "canonical_mutation": False,
        "index_rebuilt": False,
        "rollback": no_rollback(),
        "collection_tables": list(COLLECTION_TABLES),
    }
    if disposition == "applied":
        result.update(
            mode="apply",
            changed_paths=[".hive/knowledge/Wiki/release-policy.md"],
            canonical_mutation=True,
            index_rebuilt=True,
        )
    elif disposition == "rolled-back":
        result.update(mode="apply", added_count=0)
        result["rollback"] = {
            "attempted": True,
            "succeeded": True,
            "backup_digest": digest("7"),
            "restored_paths": [".hive/knowledge/Wiki/release-policy.md"],
        }
    elif disposition == "noop":
        result["added_count"] = 0
    return result


def reviewed_claim() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "claim_id": "dependency-rust-version",
        "kind": "dependency-evidence",
        "statement": "The workspace uses Rust 1.90.",
        "version": "1.90",
        "revision": None,
        "applicability": None,
        "evidence": [
            {
                "locator": "Cargo.toml",
                "content_digest": digest("8"),
                "kind": "implementation",
            }
        ],
        "agent_reviewed": True,
        "global_promotion_candidate": False,
    }


def scan_review() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "inventory_digest": digest("9"),
        "claims": [reviewed_claim()],
    }


def scan_outcome() -> dict[str, Any]:
    return {
        "canonical_target": "C:\\workspace\\project-alpha",
        "inventory": {
            "schema_version": 1,
            "root_kind": "git",
            "include_untracked": False,
            "included_count": 1,
            "skipped_count": 0,
            "included_bytes": 128,
            "entries": [
                {
                    "relative_path": "Cargo.toml",
                    "content_digest": digest("8"),
                    "byte_len": 128,
                    "tracked": True,
                    "decision": "included",
                    "reason": "dependency-manifest",
                }
            ],
            "inventory_digest": digest("9"),
            "unchanged": False,
        },
        "target_mutated": False,
    }


def validated_claims() -> dict[str, Any]:
    return {
        "inventory_digest": digest("9"),
        "collection_claims": [reviewed_claim()],
        "promotion_candidates": [],
    }


def runtime_scan_apply_result(changed: bool = True) -> dict[str, Any]:
    changed_paths = (
        [".hive/knowledge/Collections/dependency-rust-version.md"] if changed else []
    )
    return {
        "phase": "apply",
        "scan": scan_outcome(),
        "validated_claims": validated_claims(),
        "collection": {
            "collection_id": collection_id(),
            "kind": "directory",
            "state": "attached",
            "aliases": ["project-alpha"],
            "local_locator": "C:\\workspace\\project-alpha",
            "default_visibility": "project-private",
        },
        "store": {
            "changed_paths": changed_paths,
            "generation": 4,
            "manifest_digest": digest("b"),
        },
        "automatic_promotion": {
            "source_claims": [],
            "promoted_claims": [],
            "store": {
                "changed_paths": [],
                "generation": 4,
                "manifest_digest": digest("b"),
            },
        },
        "target_mutated": False,
    }


class KnowledgeSchemaConformanceTests(unittest.TestCase):
    def assert_valid(self, name: str, instance: Any) -> None:
        errors = sorted(validator(name).iter_errors(instance), key=lambda error: list(error.path))
        if errors:
            rendered = "\n".join(
                f"{list(error.absolute_path)}: {error.message}" for error in errors
            )
            self.fail(f"{name} rejected a valid fixture:\n{rendered}")

    def assert_invalid(self, name: str, instance: Any) -> None:
        self.assertFalse(validator(name).is_valid(instance), name)

    def test_all_schemas_are_draft_2020_12_valid(self) -> None:
        for name in SCHEMA_NAMES:
            with self.subTest(schema=name):
                document = schema(name)
                self.assertEqual(
                    document["$schema"], "https://json-schema.org/draft/2020-12/schema"
                )
                Draft202012Validator.check_schema(document)

    def test_every_explicit_object_and_array_is_closed_and_bounded(self) -> None:
        def visit(node: Any, location: str) -> None:
            if isinstance(node, dict):
                if node.get("type") == "object":
                    self.assertIs(
                        node.get("additionalProperties"),
                        False,
                        f"open object at {location}",
                    )
                if node.get("type") == "array":
                    self.assertIn("maxItems", node, f"unbounded array at {location}")
                for key, value in node.items():
                    visit(value, f"{location}/{key}")
            elif isinstance(node, list):
                for index, value in enumerate(node):
                    visit(value, f"{location}/{index}")

        for name in SCHEMA_NAMES:
            with self.subTest(schema=name):
                visit(schema(name), name)

    def test_contract_field_sets_and_fixed_collection_tables_are_exact(self) -> None:
        expected_properties = {
            "knowledge-collection-registry.schema.json": {"schema_version", "collections"},
            "knowledge-remember-request.schema.json": {
                "collection_id",
                "claim_key",
                "claim_id",
                "locator",
                "kind",
                "status",
                "visibility",
                "normalized_fact",
                "provenance",
                "sources",
                "supersedes",
                "expected_active_digest",
                "observed_at",
                "verified_at",
            },
            "knowledge-retrieve-request.schema.json": {
                "scope",
                "current_collection_id",
                "query",
                "query_expansions",
                "top_k",
                "byte_budget",
                "confidential_collection_id",
            },
            "knowledge-retrieval-result.schema.json": {
                "generation",
                "manifest_digest",
                "hits",
                "returned_bytes",
                "insufficient_budget",
                "search",
            },
            "knowledge-bundle-manifest.schema.json": {
                "schema_version",
                "source",
                "scope",
                "entries",
            },
            "knowledge-scan-review.schema.json": {
                "schema_version",
                "inventory_digest",
                "claims",
            },
        }
        for name, expected in expected_properties.items():
            with self.subTest(schema=name):
                self.assertEqual(set(schema(name)["properties"]), expected)

        registry = schema("knowledge-collection-registry.schema.json")
        self.assertEqual(tuple(registry["x-fixedCollectionTables"]), COLLECTION_TABLES)
        table_schema = schema("knowledge-import-result.schema.json")["$defs"][
            "collection_tables"
        ]
        self.assertEqual(
            tuple(item["const"] for item in table_schema["prefixItems"]),
            COLLECTION_TABLES,
        )

    def test_remember_provenance_source_enum_excludes_raw_runtime_material(self) -> None:
        source_kind = schema("knowledge-remember-request.schema.json")["$defs"][
            "provenance"
        ]["properties"]["source_kind"]["enum"]
        self.assertEqual(source_kind, ["user-statement", "reviewed-artifact"])
        for hostile in ("raw-transcript", "transcript", "tool-output", "hook-payload"):
            instance = remember_request()
            instance["provenance"]["source_kind"] = hostile
            self.assert_invalid("knowledge-remember-request.schema.json", instance)

    def test_collection_registry_accepts_canonical_and_rejects_hostile_shapes(self) -> None:
        name = "knowledge-collection-registry.schema.json"
        instance = collection_registry()
        self.assert_valid(name, instance)

        hostile = copy.deepcopy(instance)
        hostile["runtime_path"] = "C:\\leak"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["collections"][1]["local_locator"] = "C:\\stale"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        del hostile["collections"][1]["source_project_id"]
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["collections"][0]["collection_id"] = collection_id("b")
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["collections"][0]["aliases"] = ["x" * 257]
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["collections"].append(copy.deepcopy(hostile["collections"][0]))
        self.assert_invalid(name, hostile)

    def test_remember_request_accepts_typed_claim_and_rejects_hostile_input(self) -> None:
        name = "knowledge-remember-request.schema.json"
        instance = remember_request()
        self.assert_valid(name, instance)

        hostile = copy.deepcopy(instance)
        hostile["status"] = "superseded"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["locator"] = "../outside.md"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["claim_id"] = "claim-not-a-digest"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["raw_tool_output"] = "not canonical provenance"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["normalized_fact"] = "x" * 16385
        self.assert_invalid(name, hostile)

    def test_retrieve_request_enforces_serde_scope_budget_and_exact_authorization(self) -> None:
        name = "knowledge-retrieve-request.schema.json"
        instance = retrieve_request()
        self.assert_valid(name, instance)
        for scope in ("auto", "global", "all-visible", {"project": "project-alpha"}):
            candidate = copy.deepcopy(instance)
            candidate["scope"] = scope
            self.assert_valid(name, candidate)

        hostile = copy.deepcopy(instance)
        hostile["include_confidential"] = True
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["scope"] = f"collection:{collection_id()}"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["top_k"] = 0
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["query_expansions"] = [str(index) for index in range(9)]
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["confidential_collection_id"] = "friendly-alias"
        self.assert_invalid(name, hostile)

    def test_retrieval_result_requires_stable_citations_and_untrusted_marker(self) -> None:
        name = "knowledge-retrieval-result.schema.json"
        instance = retrieval_result()
        self.assert_valid(name, instance)

        hostile = copy.deepcopy(instance)
        hostile["hits"][0]["untrusted_content"] = False
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["hits"][0]["item_kind"] = "document"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["hits"][0]["chunk_id"] = "chunk-unstable"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["returned_bytes"] = 1048577
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["hits"][0]["instructions"] = "execute me"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["hits"][0]["scan_metadata"]["promotion_status"] = "not-candidate"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["hits"][0]["scan_metadata"]["evidence"][0]["locator"] = "../secret"
        self.assert_invalid(name, hostile)
        no_source = copy.deepcopy(instance)
        no_source["hits"][0]["sources"] = []
        self.assert_valid(name, no_source)

    def test_bundle_manifest_allows_only_bounded_portable_payloads(self) -> None:
        name = "knowledge-bundle-manifest.schema.json"
        instance = bundle_manifest()
        self.assert_valid(name, instance)

        for classification in (
            "derived-sqlite",
            "runtime-state",
            "absolute-path",
            "credential",
            "confidential",
        ):
            hostile = copy.deepcopy(instance)
            hostile["entries"][0]["classification"] = classification
            self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["entries"][0]["path"] = "data/../secrets.txt"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["entries"][0]["path"] = "data/Wiki/CON.md"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["entries"][0]["length"] = 8388609
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["source"]["id"] = "C:\\machine-bound"
        self.assert_invalid(name, hostile)

    def test_import_result_distinguishes_successes_and_rejects_rolled_back_failure(self) -> None:
        name = "knowledge-import-result.schema.json"
        for disposition in ("planned", "noop", "applied"):
            with self.subTest(disposition=disposition):
                self.assert_valid(name, import_result(disposition))
        disabled_applied = import_result("applied")
        disabled_applied["index_rebuilt"] = False
        self.assert_valid(name, disabled_applied)

        rolled_back = import_result("rolled-back")
        self.assert_invalid(name, rolled_back)
        detached_contract = schema(name)["properties"]["detached_collection_ids"]
        self.assertEqual(detached_contract["maxItems"], 10000)
        many_detached = import_result("planned")
        many_detached["detached_collection_ids"] = [
            f"collection-{index:064x}" for index in range(257)
        ]
        self.assert_valid(name, many_detached)

        hostile = import_result("planned")
        hostile["canonical_mutation"] = True
        self.assert_invalid(name, hostile)
        hostile = import_result("applied")
        hostile["changed_paths"] = []
        self.assert_invalid(name, hostile)
        hostile = import_result("applied")
        hostile["added_count"] = 0
        self.assert_invalid(name, hostile)
        hostile = import_result("planned")
        hostile["collection_tables"][3] = "semantic_cache"
        self.assert_invalid(name, hostile)
        hostile = import_result("noop")
        hostile["rollback"]["backup_digest"] = digest("b")
        self.assert_invalid(name, hostile)

    def test_scan_review_requires_agent_review_and_typed_evidence(self) -> None:
        name = "knowledge-scan-review.schema.json"
        instance = scan_review()
        self.assert_valid(name, instance)

        hostile = copy.deepcopy(instance)
        hostile["claims"][0]["agent_reviewed"] = False
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["claims"][0]["version"] = None
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["claims"][0]["evidence"][0]["locator"] = "../Cargo.toml"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["claims"][0]["evidence"][0]["locator"] = "docs/./Cargo.toml"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(instance)
        hostile["claims"][0]["transcript"] = "raw conversation"
        self.assert_invalid(name, hostile)

        preference = reviewed_claim()
        preference.update(kind="preference", version=None)
        hostile = scan_review()
        hostile["claims"] = [preference]
        self.assert_invalid(name, hostile)

    def test_scan_result_covers_all_runtime_phases(self) -> None:
        name = "knowledge-scan-result.schema.json"
        inventory = {"phase": "inventory", "scan": scan_outcome()}
        candidates = {
            "phase": "candidates",
            "scan": scan_outcome(),
            "validated_claims": validated_claims(),
            "canonical_mutation": False,
        }
        for instance in (
            inventory,
            candidates,
            runtime_scan_apply_result(),
            runtime_scan_apply_result(changed=False),
        ):
            self.assert_valid(name, instance)

        hostile = copy.deepcopy(inventory)
        hostile["scan"]["inventory"]["entries"][0]["content_digest"] = None
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(inventory)
        hostile["scan"]["inventory"]["entries"][0]["content"] = "raw bytes"
        self.assert_invalid(name, hostile)
        hostile = copy.deepcopy(candidates)
        hostile["canonical_mutation"] = True
        self.assert_invalid(name, hostile)
        hostile = runtime_scan_apply_result()
        hostile["collection"]["source_project_id"] = "project-alpha"
        self.assert_invalid(name, hostile)
        hostile = runtime_scan_apply_result()
        hostile["store"]["generation"] = 0
        self.assert_invalid(name, hostile)
        hostile = runtime_scan_apply_result()
        hostile["disposition"] = "applied"
        self.assert_invalid(name, hostile)
        hostile = runtime_scan_apply_result()
        hostile["rollback"] = no_rollback()
        self.assert_invalid(name, hostile)


if __name__ == "__main__":
    unittest.main()
