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
  - "repo:docs/architecture/vector-search.md#sha256:7200f812a0d660d5740ccf0cd656095e0f266de8f120a1ba4738010a6f940d2a"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/guides/vector-search.md#sha256:0c3fe94600b5ec85ff34dcc0eec814a9d4fabca1772ffc5021c5d97478dbfd0d"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:413f77fcba2a2ba20071aef4f8a0ac77582ed11e219c750dd312cbab189b3e9c"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:fb2acff2d082e4b87fc89cc5a3979bc276f1b80e84b2a8a9e589b638adabb743"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:c6f1663011110ebe7a09f655e2e2f663083be8af"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

Markdown stays canonical; FTS remains compatible. Local embedding needs consent; confidential
query and build approvals are separate. MiniLM passes both semantic recall sets. The latest
Windows fresh build of 50,000 chunks in 100 collections takes 638.597 seconds and fails the time
gate. Vectors are exact and storage is 442.8 MB. The latest 100-change test fails at 51.989 seconds.
One CPU class, shared models, parallel canonical checks and EOF-preserving buffers are implemented.
Final query measurements remain required. Fresh authority and byte checks remain.
Literal FTS order protection removes six rank losses; exact MRR stays 0.975. Stable needs approval.
