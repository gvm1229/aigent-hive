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
  - "repo:docs/architecture/vector-search.md#sha256:fa3b27aa0e59bd3931855426e24bdc6f7bc78aff99bd2adf3cae64928af7569e"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/guides/vector-search.md#sha256:e24d70629bf128f5dddfbe947e52841b6d71bdcf241d65527b420acf6bc93e54"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:8c1a2188796d37696736b87097f07cd54b3a311591650770af32a2a795e58634"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:5a7feb09d93de460c036ed35e2b90a5620168c51b60ec58747510dec4f00bd48"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:c264d0e315249dc2f95a58b2c4ab02375b0acad4"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

The vector branch preserves canonical Markdown and FTS. Local embedding needs consent;
confidential query and build approvals remain separate. MiniLM passes fixed original and
independent question gates. A
Windows P-core experiment builds 50,000 chunks across 100 collections in 532.66 seconds with
372.2 MB including the prior generation. Mixed P/E execution reproduces numeric differences;
the worker now selects one permitted CPU class. Incremental lock contention and whole-query
latency remain failed gates. Component timing never proves whole-query speed. Final-profile
qualification and explicit stable-release approval remain required. Bounded shared rebuild
windows and single-query SQLite connection reuse retain fresh authority and byte checks.
