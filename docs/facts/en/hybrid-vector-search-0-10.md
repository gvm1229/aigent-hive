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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:3e669b1c196d9176fdc908766d00700be7b30e43e8f53a5f72c1f2d178d44016"
  - "repo:docs/guides/vector-search.md#sha256:5e8e1cbda784d7e1c9acb7c1187d423ec6adadc090ca9fb2c64045afc2426dc0"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:337e9c34ef9bdfd072b384dfb46affb7143c129a4f7c0083d7720e6ba7d5f4cc"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-acceptance-2026-08-29.md#sha256:ae22c31066420d6e2a22e04febe336b661ca5021228e7c979badda3190199388"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:44bacc8fad38a290054a8749d68c83f59d794a53df2739228acd27805bd12e11"
  - "repo:docs/research/vector-public-test6-2026-08-29.md#sha256:b9e4e73e31ea0b5df6c1c419400e477fdc11e042b8d95d0b5f94c2baac48ab6b"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:d331dc879cf51eab078c5e189b2fe7b8d729e541"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

Markdown stays canonical; FTS remains available. Local embedding needs consent; confidential
query and build approvals are separate. The maintainer accepted sqlite-vec and MiniLM stress
performance on 2026-08-29. Both semantic sets score 58/60; exact ranks and 50,000 vectors are
preserved. Storage passed; linked sources retain detailed measurements and prior failures.
Quality, safety and recovery gates remain unchanged. Retained-model execution is omitted.
Public test.6 passed installation and vector acceptance on three operating systems. Fresh-root
import and rebuild were tested, not transfer between physical computers. Stable needs explicit approval.
