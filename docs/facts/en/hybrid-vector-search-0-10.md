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
  - "repo:docs/architecture/vector-search.md#sha256:ba354b18c8f8f4940d0387c002e904af43dc31d25aa9c72ef7541e9ba0f463bb"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/guides/vector-search.md#sha256:f39152f4aa591b4e8425ba7ebc3fe0274e4853c13eeb0605ebeac0fcfaa3674c"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:413f77fcba2a2ba20071aef4f8a0ac77582ed11e219c750dd312cbab189b3e9c"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:ed0e299d465593ea0030e796b015a37c5a243914d9028ca1551769e70b9ae498"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:f1f1c04c2fde76c5426a1e7a91c22b68c8e87753"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

The vector branch preserves canonical Markdown and FTS. Local embedding needs consent;
confidential query and build approvals remain separate. MiniLM passes original and independent
semantic recall gates. Windows parallel builds of 50,000 chunks in 100 collections pass at 577.810
seconds with exact vectors and 442.8 MB. The parallel 100-change test fails at 56.313
seconds. One CPU class, shared models and parallel publication checks are implemented.
Bounded read buffers preserve EOF checks. Final query and rebuild measurements remain required.
Fresh authority, canonical and byte checks remain. Stable release needs explicit approval.
Literal FTS order protection removes all six exact-rank losses; exact MRR stays at 0.975.
