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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:849e3187ee34c5dc9b68740c5fb269a9dd3988ba67bb0d46a105796cbd490061"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:4f3d3dbb7d7b4e456dfb827f116e5f68ce113834c28bfcaaec0fcd3e377ea1a3"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:3b5414a55284020a2a85ad2c26b7fd81a3b053247aaa07bdfb42a2dc208906fa"
  - "repo:crates/hive-wiki/src/store.rs#sha256:5b3bc23d06794f76d40f879644e621a8132f5c177f052c2e314205cd7e5c80bd"
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
