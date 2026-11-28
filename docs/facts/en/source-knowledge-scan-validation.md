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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:849e3187ee34c5dc9b68740c5fb269a9dd3988ba67bb0d46a105796cbd490061"
  - "repo:crates/hive-wiki/src/store.rs#sha256:5b3bc23d06794f76d40f879644e621a8132f5c177f052c2e314205cd7e5c80bd"
links: [knowledge-cross-project-access, knowledge-portability-scan, source-development]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Reviewed Scan Validation Parity

`hive knowledge scan --candidates` and `--apply` now run the same reviewed-claim credential
validation before registry or index mutation. A rejected claim identifies its reviewed claim ID and
statement field instead of incorrectly blaming raw source material.

Canonical scan provenance keeps the review ID in typed metadata and omits it from the human summary.
This prevents an ordinary descriptive ID from being treated as an opaque credential during canonical
claim verification. Source claims remain project-private; explicit collection retrieval is required.
