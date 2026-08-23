---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: en
counterpart: ../ko/hybrid-vector-search-0-10.md
title: "Hybrid Vector Search Gate for 0.10.0"
summary: "0.10.0 vector requalification deferred the optional adapter because the unique 50,000-chunk embedding build failed the ten-minute gate."
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:9870204c4032c4c43504b73d20689b2104eba5d8ff826b607016866fd22155b5"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:4044da7cd06e38d0d3da6f7640c4318ecc09fa077acc3b5298ccc3b46f0e8612"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:571467bb776b86bed509a06cdb6744434b067993"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

Requalification kept the 15-point semantic Recall@10 gain, 100% hybrid exact recall, and fast query
embedding. Digest reuse built 50,000 repeated rows from 30 embeddings in 5.75 seconds. A 1,000-row
probe projected about 2,711 seconds for 50,000 unique chunks, above the 600-second gate. Resumable
and incremental research paths passed, but the optional adapter stays deferred. FTS and graphs stay
active. No vector product dependency exists.
