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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:cb4e53566ee902c209f8e0593929b05b8c5d22b87128736c638f4aa57d7b0f21"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a0cf1b55b6cfc9d02920f117d25fb752e25267093d1be960b33267b8f6b0a94c"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:631d823968b135efa90bb0515185fed724c1c1522af3623aa1a26e7c9842d1a7"
  - "repo:crates/hive-wiki/src/store.rs#sha256:812c302031f36694ad109479698f9d89b125392dba55468c0131e52d45f47b44"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:aaea806d5543632cf6ebf6c9faee349e6d67a97f9b8c8bd753bd0da9e02582b7"
  - "repo:harness/skills/knowledge-capture/references/scan.md#sha256:83063c0c0a97e81bd71170c1325df6a118d21e0659a806eecd6723912c1b26f0"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:d80f5871428941be1d05b12d488857f40bb894752c0c4354cee12f15a46578f6"
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
