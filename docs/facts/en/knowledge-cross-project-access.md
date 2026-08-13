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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:c61df64b6ae0367c4e146346472a108d22576e6f8eee07e8581b29d7ffe25784"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:15a09e0b770055a0cdab1191048d53c0323a892e2ba8eb374d4bf30cb5491c13"
  - "repo:crates/hive-wiki/src/store.rs#sha256:d2933a18fede81c05988727a07f5b9de019d4f488cce4d0d8876113d2be1d4b2"
  - "repo:harness/skills/knowledge-import/SKILL.md#sha256:c20be7748412c966c9fe87d6a97281ac7eb00381607b4443d2cfe555c07e01f3"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:7b5d334b67e9db1b981273f9fb134adca8129250f241d7359f6f1bc5bda88c1e"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:6d5798e1a4ed03a79f0d97ed596d3229121af5e8"
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
