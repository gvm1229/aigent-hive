---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: en
counterpart: ../ko/hybrid-vector-search-0-10.md
title: "Hybrid Vector Search Gate for 0.10.0"
summary: "0.10.0 compares FTS, vector, and hybrid retrieval and implements an optional local vector adapter only after quality, latency, security, and portability gates pass."
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:a475f0b665f301e2398506e2f3a16d2678f83eb2ad8dc6a72f7d4c522673b409"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:c608753d9238fa9002e33f69b8558e1433aecc467db5cdf8c946a0dbfe3b9442"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:e8580fffed7ee2ea0e123f4171bc7a03e7ae5444faedfa5e9bf1fac5796475d7"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
status: active
---

# Hybrid Vector Search Gate for 0.10.0

`KRG10-014` and `VEC10-001–012` compare FTS, Qdrant Edge, and SQLite vector engines on a
120-query gold corpus and 50,000 chunks. Adoption requires semantic Recall@10 improvement, exact
fact non-regression, bounded latency and storage, offline scope isolation, and a pinned local
non-generative embedding indexer. Failure adds no product dependency and preserves FTS and graphs.
