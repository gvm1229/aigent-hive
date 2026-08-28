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
  - "repo:docs/architecture/vector-search.md#sha256:b7787399d3039ec0ad833b8f7c527780e679a42b9172df968dc25495a038e085"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:d63afd36e3ebcd3145f77c24a6dd719be2216e458db791dbffe583dd5781c9c6"
  - "repo:docs/guides/vector-search.md#sha256:f5956f67785ca9bff90a4286c13993616daeddd7b9cbc43eaf6a34582233b64b"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:8c1a2188796d37696736b87097f07cd54b3a311591650770af32a2a795e58634"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:9e68d17bb961811dd74d025a72551b12dcf8b4d66c87deb63559e1c579ad04f7"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:ce2169b55073e5564dcbe8c78a9660f2b7efc816"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

The vector branch preserves canonical Markdown and FTS. Local embedding needs consent;
confidential query and build approvals remain separate. Three-OS native functional checks pass
before the latest CPU fix. MiniLM passes fixed original and independent question gates. A
Windows P-core experiment builds 50,000 chunks across 100 collections in 532.66 seconds with
372.2 MB including the prior generation. Mixed P/E execution reproduces numeric differences;
the worker now selects one permitted CPU class. Incremental lock contention and whole-query
latency remain failed gates. Component timing never proves whole-query speed. Final-profile
qualification and explicit stable-release approval remain required.
