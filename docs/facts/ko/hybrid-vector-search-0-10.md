---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: ko
counterpart: ../en/hybrid-vector-search-0-10.md
title: "0.10.0 Hybrid vector search gate"
summary: "50,000개 고유 chunk embedding의 10분 gate 실패로 선택형 vector adapter를 보류한 0.10.0 재검증 결과"
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:921f2847dacea259c29b9f6c8cbb2c4f7c090429e04771ec240d49eb1ccfbb72"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:df7502f8bf610d13f4269d5cbd344857157325ab56d34154f154dbfb7b730364"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:571467bb776b86bed509a06cdb6744434b067993"
status: active
---

# `0.10.0` hybrid vector search gate

재검증 결과: semantic Recall@10 `+15.0 points`, hybrid exact recall `100%`, 빠른 query
embedding 유지. Digest 재사용 기반 반복 50,000건 build: 30 embeddings·5.75초. 고유 1,000건
probe 기반 50,000건 환산값: 약 2,711초, 600초 gate 실패. 중단 재개·증분 연구 경로 통과.
선택형 adapter 보류, FTS·graph 유지, vector product dependency `0건`.
