---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: en
counterpart: ../ko/hybrid-vector-search-0-10.md
title: "Hybrid Vector Search Gate for 0.10.0"
summary: "0.10.0 vector implementation reopened on its own branch while the original quality, performance, and safety gates remain required."
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/architecture/vector-search.md#sha256:4789dac545a49436d777c5bffa28b31cbb0e47a8bb34396363a214b9d2ebeeb8"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:08b5950d158c3e374752b625ba93a715d44c990d32c2ef39490bef2a1b9b084d"
  - "repo:docs/guides/vector-search.md#sha256:031e3db0bfd3dddf932a012bff98ed213dcb5a542f39921813c41acd109b5fb3"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:de7fb2f8aadab112f3b86da075fd972a4edcf999787eaf7585edcfe1a9fab4a9"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:32547d3a1332b6da5366bbeb9b95c968bb57cb83b115bbd563181ff676465847"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:d9cb7733237df4e5cc14824cb2df13ff75009776"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

Markdown stays canonical; FTS remains available. Local embedding needs consent; confidential
query and build approvals are separate. Both semantic sets score 58/60. Exact gold MRR stays
0.975; all 60 numbered semantic lookups preserve FTS rank 1. Windows fresh build takes 638.597s,
100-change rebuild 51.989s, global/collection query p95 3.067/2.276s, and 100DB SQL lookup p95
98.876ms: all miss their time gates. The 50,000 vectors remain exact; storage is 442.8 MB.
Functions are verified, but adoption and test.5 remain pending. Task-scoped model retention is
an unapproved proposal. Fresh authority and byte checks remain. Stable needs explicit approval.
