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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:f0e47ded9439c9d2fcb2c1be6eb93d11609e942d5320f452fd45feecc7bf7d8a"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:2bb27720c34a60bfd3b0003e27348288f3f17062ab1e270f3c2d624487e1eff4"
  - "repo:crates/hive-wiki/src/store.rs#sha256:6d6a377a6cd0c0c38ca48a85e89e871210ef4e87bbe05cf80c17713a566ae9a0"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:9e169f3daff2b4fbe6cff4d9a93d7e45cca6e9a6e78d1784b83458b50d3aa267"
  - "repo:harness/skills/knowledge-scan/SKILL.md#sha256:b8c3928df97c6f5e84f60b5a20ed9944c3ccd785cc408ffa5aa4a1db4d4b2aef"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
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
