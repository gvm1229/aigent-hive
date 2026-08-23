---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: ko
counterpart: ../en/hybrid-vector-search-0-10.md
title: "0.10.0 Hybrid vector search gate"
summary: "50,000 document offline embedding build의 10분 초과로 optional vector adapter를 defer한 0.10.0 판정"
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:ff4dfde9029c9024ab260f0366381e1a9bf1ce9d384a1db46b33d1cd842a5578"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:0c1da49e94d8865101bfe30c2f95919c8f562ec6aa301118f02b1ed5bc79ffdd"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
status: active
---

# `0.10.0` hybrid vector search gate

- 품질: Semantic Recall@10 `+15.0 points`, hybrid exact 질문 `100%`
- Engine: Qdrant Edge·sqlite-vec·SQLite-Vector 50,000 lookup·storage 기준 통과
- 실패: Windows x64 offline embedding full build 10분 초과
- 추가 제외: SQLite-Vector의 비공개 상용 소비자 license 부적합
- 판정: `defer`
- Product dependency: Vector engine·embedding runtime·model·schema `0건`
- 기존 기능: FTS·native graph·Graphify code adapter 유지
