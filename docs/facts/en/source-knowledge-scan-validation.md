---
schema_version: 1
pair_id: source-knowledge-scan-validation
topic_slug: source-knowledge-scan-validation
language: en
counterpart: ../ko/source-knowledge-scan-validation.md
title: "Reviewed Scan Validation Parity"
summary: "Candidate and apply share credential validation, and scan provenance excludes human review IDs from its rendered summary."
tags: [knowledge, scan, source, v0-9-4, validation]
aliases: ["Reviewed source import", "Scan validation parity"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:fb2a64a7cafaf921a5252a65b042ade24ab03b7dcee5eec52dc7b895b5b92df5"
  - "repo:crates/hive-wiki/src/store.rs#sha256:f9f3dcf6627b0c1a2e08a05596f2e027c9a75c9fbbcafb48fbe8d29c000e1481"
links: [knowledge-cross-project-access, knowledge-portability-scan, source-development]
reviewed_revision: "git:52e63238cb6063312240d1c9b5ed1006d8a23a7b"
status: active
---

# Reviewed Scan Validation Parity

`hive knowledge scan --candidates` and `--apply` now run the same reviewed-claim credential
validation before registry or index mutation. A rejected claim identifies its reviewed claim ID and
statement field instead of incorrectly blaming raw source material.

Canonical scan provenance keeps the review ID in typed metadata and omits it from the human summary.
This prevents an ordinary descriptive ID from being treated as an opaque credential during canonical
claim verification. Source claims remain project-private; explicit collection retrieval is required.
