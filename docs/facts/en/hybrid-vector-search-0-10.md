---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: en
counterpart: ../ko/hybrid-vector-search-0-10.md
title: "Hybrid Vector Search Gate for 0.10.0"
summary: "0.10.0 defers the optional vector adapter because the 50,000-document offline embedding build exceeded ten minutes."
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:0c1da49e94d8865101bfe30c2f95919c8f562ec6aa301118f02b1ed5bc79ffdd"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

The 120-query corpus showed a 15-point semantic Recall@10 gain with a local multilingual model.
Qdrant Edge, sqlite-vec, and SQLite-Vector passed the 50,000-vector lookup and storage budgets.
The offline embedding build exceeded the ten-minute Windows x64 gate, and SQLite-Vector also has
an incompatible closed-commercial-use license boundary. The `0.10.0` decision is `defer`, with no
vector engine, embedding runtime, model, or schema product dependency. FTS and graphs remain active.
