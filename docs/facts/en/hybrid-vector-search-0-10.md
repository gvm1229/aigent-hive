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
  - "repo:docs/architecture/vector-search.md#sha256:5f87678612089299a881e50918dd3565ba8e37126351d4aa95c24553f53f0284"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:d63afd36e3ebcd3145f77c24a6dd719be2216e458db791dbffe583dd5781c9c6"
  - "repo:docs/guides/vector-search.md#sha256:f5956f67785ca9bff90a4286c13993616daeddd7b9cbc43eaf6a34582233b64b"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:2b2e773f3b2686f9e49ac84392aff14e7c613adc19a198d7b7562affe512124a"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:55591c7c31eed28a9ebd16db42f313ee3723548ba79aff16913db22b21d1240e"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:ce2169b55073e5564dcbe8c78a9660f2b7efc816"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

The vector branch follows the non-vector repairs merged into `develop`. Markdown remains
canonical and FTS available. Local non-generative embedding needs consent; confidential query
and build use separate one-time approval. Windows native recovery, long paths, canonical-only
import and source queries pass. MiniLM score fusion passes the fixed question gates; the shared
CPU helper builds 50,000 chunks within ten minutes. The full 100-collection CLI baseline still
fails time, storage and numeric equivalence gates. Report the original 120 and independent 60
questions separately; no success erases a failed gate. Embedding-only timing does not prove full
query latency. Stable release requires explicit approval.
