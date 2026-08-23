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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:72af50e7f2498159a60ea4a90f7e77fd96fd712506922ec683eeecbcae4f889a"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:5fc5feca2ab250f5f91c8d3a678c315fabe4fdcf12763fb664a0a6b4bcbe0c81"
  - "repo:crates/hive-wiki/src/store.rs#sha256:39f62b339764e470446c61bfb392b2f8637908738261c8fe5bc9b711da0bb40d"
  - "repo:harness/skills/knowledge-import/SKILL.md#sha256:b73e6c82eb5ef9105781383f59211a982ed70b0f0ecf1690619f3b9b30f4730d"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:531437bfcb9786cd5221de32eb5ad536bfd07973db159ca0b15a5df858ffa923"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
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
