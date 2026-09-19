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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:0ffab09ca1eaac47b41608e04003fdbc51ee7e5edbaf2386554c885d9f55ec58"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:78a309398bc924b72c98eb522ac4ef7d944f172f769eab72d8b8916444c257cc"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:2bb27720c34a60bfd3b0003e27348288f3f17062ab1e270f3c2d624487e1eff4"
  - "repo:crates/hive-wiki/src/store.rs#sha256:a804c6475388324fe8e785f4afdb852a3318cfc4ba2e2fba1c9ae8eb313d2e20"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:6eed0fc1dfcee4d063ba6e3877e0ae2043dc36b8abad8b358551ad1515ddc57e"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:7ca8ab883c9dc93c06e585f75be3a4ff0dd2ee0dc6c9adcbf68cf5798a43a416"
  - "repo:harness/skills/knowledge-scan/SKILL.md#sha256:b8c3928df97c6f5e84f60b5a20ed9944c3ccd785cc408ffa5aa4a1db4d4b2aef"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:dd63333a702a7a89585d101d2b9d043ebd0987d8"
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
