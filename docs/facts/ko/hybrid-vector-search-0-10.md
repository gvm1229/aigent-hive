---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: ko
counterpart: ../en/hybrid-vector-search-0-10.md
title: "0.10.0 Hybrid vector search gate"
summary: "FTS·vector·hybrid 검색 비교 뒤 품질·latency·보안·이식성 gate 통과 조합만 optional local vector adapter로 구현하는 0.10.0 계약."
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:fe327177fca73ccbdb3267a1cfca7b579b984e8bd3a24e74457a7d062020f2ec"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:c608753d9238fa9002e33f69b8558e1433aecc467db5cdf8c946a0dbfe3b9442"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:e8580fffed7ee2ea0e123f4171bc7a03e7ae5444faedfa5e9bf1fac5796475d7"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
status: active
---

# `0.10.0` hybrid vector search gate

- 비교: FTS·Qdrant Edge·SQLite vector engine, gold query `120개`, chunk `50,000개`
- 채택: semantic Recall@10 향상, exact fact 무회귀, latency·storage·scope gate 통과
- Embedding: pinned local non-generative indexer, provider API·API key `0건`
- 실패: vector product dependency `0건`, 기존 FTS·graph 일정 유지
