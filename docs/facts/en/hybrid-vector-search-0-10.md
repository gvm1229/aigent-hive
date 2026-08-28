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
  - "repo:docs/architecture/vector-search.md#sha256:91883e7c324dab8bc8beb7cfe1f39a7cde6ddef9024dbd873b1b7adb52e0dd54"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/guides/vector-search.md#sha256:e24d70629bf128f5dddfbe947e52841b6d71bdcf241d65527b420acf6bc93e54"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:8c1a2188796d37696736b87097f07cd54b3a311591650770af32a2a795e58634"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:e1702b22a067e7e53905f643f2a0ba49687fd00a8d57c7dc901ae62e222b46ac"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:ddb91859aae17ee52c79ca2b14fdaebb5f2876dd"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

The vector branch preserves canonical Markdown and FTS. Local embedding needs consent;
confidential query and build approvals remain separate. MiniLM passes original and independent
semantic recall gates. Windows parallel builds of 50,000 chunks in 100 collections pass at 577.810
seconds with exact vectors and 442.8 MB. The earlier serial 100-change test fails at 95.943
seconds. One CPU class, shared models and parallel publication checks are implemented.
New incremental and query measurements remain required. Component timing does not prove query latency.
Fresh authority, canonical and byte checks remain. Stable release needs explicit approval.
Forced semantic mode lowers six exact-answer ranks; default FTS preservation is a separate claim.
