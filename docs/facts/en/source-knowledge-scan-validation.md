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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:f3f69d5fcea2bd8fb6383b29ae97d490e79ff794345826c0f68c550fb5881db4"
  - "repo:crates/hive-wiki/src/store.rs#sha256:39f62b339764e470446c61bfb392b2f8637908738261c8fe5bc9b711da0bb40d"
links: [knowledge-cross-project-access, knowledge-portability-scan, source-development]
reviewed_revision: "git:bd525f15a1cb2e96eef3e90d7bec1441c52a136c"
status: active
---

# Reviewed Scan Validation Parity

`hive knowledge scan --candidates` and `--apply` now run the same reviewed-claim credential
validation before registry or index mutation. A rejected claim identifies its reviewed claim ID and
statement field instead of incorrectly blaming raw source material.

Canonical scan provenance keeps the review ID in typed metadata and omits it from the human summary.
This prevents an ordinary descriptive ID from being treated as an opaque credential during canonical
claim verification. Source claims remain project-private; explicit collection retrieval is required.
