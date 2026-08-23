---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: en
counterpart: ../ko/hybrid-vector-search-0-10.md
title: "Hybrid Vector Search Gate for 0.10.0"
summary: "0.10.0 reopens vector qualification with deduplication, unique chunks, resumable embedding, end-to-end latency, isolation, rollback, and three-platform gates."
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:b88eaf08d187d6f83cfac8b9e3a186791f08b71d0d5287f5dafe4d2e7aaa8151"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:1010a111e32833722e0c00b9bda1de421b21e98a025366a856dea661f2ec8ad9"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:e8bbe0529513df56e73f84cf5797bb334f4184ec"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

The first gate found a 15-point semantic Recall@10 gain and fast vector engines, but a naive
50,000-item embedding build exceeded ten minutes. Requalification now separates repeated synthetic
data from 50,000 unique chunks and adds digest reuse, resumable and incremental builds, query
embedding latency, physical scope isolation, atomic generations, rollback, and three-platform
evidence. No engine is preselected. One optional hybrid adapter may ship only if every gate passes;
otherwise FTS and graphs remain active with no vector product dependency.
