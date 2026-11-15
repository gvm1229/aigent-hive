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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:6b4d501127b0ba69d8d40d07cb57da9992ca34380d7c728f17e39deb9b6b11ee"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:35c9814239dde8505142300ca75f046ed17ac6c36e717a70bb2c358c312ee239"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:daec035ff6b447bbcb64ca2e9dc9b8d97f286d0622bee15d324a681aea7f3bf8"
  - "repo:crates/hive-wiki/src/store.rs#sha256:2d8660c5455ba2a287a96e93a4817db413338a8bf46cfb6f034108ae23ab670b"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:aaea806d5543632cf6ebf6c9faee349e6d67a97f9b8c8bd753bd0da9e02582b7"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:2233fdf8eca297da93e9ae5785598ba1d96fcf84e39ebf44cc9cfa212a23e556"
  - "repo:harness/skills/knowledge-scan/SKILL.md#sha256:b8c3928df97c6f5e84f60b5a20ed9944c3ccd785cc408ffa5aa4a1db4d4b2aef"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
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
