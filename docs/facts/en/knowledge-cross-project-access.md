---
schema_version: 1
pair_id: knowledge-cross-project-access
topic_slug: knowledge-cross-project-access
language: en
counterpart: ../ko/knowledge-cross-project-access.md
title: "Explicit Cross-Project Knowledge Access"
summary: "Automatic retrieval stays isolated; explicit collection retrieval is direct; reviewed safe-general scan claims are promoted during apply."
tags: [collection, knowledge, promotion, retrieval, v0-9-3]
aliases: ["Automatic knowledge promotion", "Cross-project knowledge"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:1dccff4126427eee75e24cb440139c4791b1fae34c115cee69d2cb46dd182e87"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a72952ecc2423f82bd6710e9d88a88cd6b59f0d87500b30c1a1d68151879bcf1"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:333781bbf41580900921ee3fa3d890f11fc04e1bad756f040e20786183b74957"
  - "repo:crates/hive-wiki/src/store.rs#sha256:f9f3dcf6627b0c1a2e08a05596f2e027c9a75c9fbbcafb48fbe8d29c000e1481"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:aaea806d5543632cf6ebf6c9faee349e6d67a97f9b8c8bd753bd0da9e02582b7"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:b1c993c5c9332596f2f71f0db2e40f42a434e210f71f4e8bb9f38da763c4b673"
  - "repo:harness/skills/knowledge-scan/SKILL.md#sha256:b8c3928df97c6f5e84f60b5a20ed9944c3ccd785cc408ffa5aa4a1db4d4b2aef"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:1cdebf8aef9dde963f5aa88a36042186fbf37e80"
status: active
---

# Explicit Cross-Project Knowledge Access

Automatic retrieval in Project A uses only A, `user-root`, and verified shared knowledge. It
does not include Project B private knowledge.

When the user explicitly names Project B or a unique collection alias, Hive resolves that
reference fail-closed and queries B directly. The result excludes A, `user-root`, and unrelated
shared collections. Confidential content retains its exact-query authorization requirement.

Reviewed safe-general decisions, conventions, and workflows with explicit applicability are
promoted automatically during scan apply and rescan maintenance. The transaction records source
provenance and promotion status; stale source evidence invalidates its derived shared claim.
Retrieval never causes promotion.
