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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:1229cfa84e1fb0357c943fd0ef2910f3cdb5dd7e70f67879f0832db0ea26c800"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:3c8553ae33ef2b26ee9a97cb28bba9859f0706def788335e690818d22fb3efb5"
  - "repo:crates/hive-wiki/src/store.rs#sha256:d49438b3d49f9ca1ac5eb574f94309846c2b9a46225704f4753dcef737881653"
  - "repo:harness/skills/knowledge-import/SKILL.md#sha256:b73e6c82eb5ef9105781383f59211a982ed70b0f0ecf1690619f3b9b30f4730d"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:531437bfcb9786cd5221de32eb5ad536bfd07973db159ca0b15a5df858ffa923"
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
