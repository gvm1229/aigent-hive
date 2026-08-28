---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: en
counterpart: ../ko/hybrid-vector-search-0-10.md
title: "Hybrid Vector Search Gate for 0.10.0"
summary: "0.10.0 local vector search accepted under the maintainer-approved stress policy; quality and safety gates stay unchanged."
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/architecture/vector-search.md#sha256:78f3d2b3ca955dd6cc48f6b926b2af10d8349a243079fd4636b3718d12b22035"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:a00c93539ccde3105ec332ac6fcb40fdf9d43e580b702e1f7035cfc6cb36d088"
  - "repo:docs/guides/vector-search.md#sha256:e5bfc863323fc0a1f6d0ce31e877ff7cd7e67a71c1e52ec4d14151cded3d51f0"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:0133d82395c24813489c56365577cb141e5974d4dc9b33b7af555bbfb8b13981"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-acceptance-2026-08-29.md#sha256:3738689349427030eb5baa80417b4f17d29a26182348b45cd3220e5bdc4efc21"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:44bacc8fad38a290054a8749d68c83f59d794a53df2739228acd27805bd12e11"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:bd16c1cd1ad598a70b1719ca466d103af12df2d8"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

Markdown stays canonical; FTS remains available. Local embedding needs consent; confidential
query and build approvals are separate. Both semantic sets score 58/60. Exact gold MRR stays
0.975; all 60 numbered semantic lookups preserve FTS rank 1. Windows fresh build takes 638.597s,
100-change rebuild 51.989s, global/collection query p95 3.067/2.276s, and 100DB SQL lookup p95
98.876ms. The maintainer accepted this stress performance on 2026-08-29 under revised limits.
Original failures remain recorded. Quality, safety and recovery gates remain unchanged.
The 50,000 vectors remain exact; storage is 442.8 MB. Retained-model execution is omitted;
test.5 acceptance remains required. Stable needs explicit approval.
