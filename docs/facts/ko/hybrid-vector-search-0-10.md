---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: ko
counterpart: ../en/hybrid-vector-search-0-10.md
title: "0.10.0 Hybrid vector search gate"
summary: "기존 품질·속도·안전 기준을 유지하면서 전용 브랜치에서 다시 진행하는 0.10.0 벡터 구현"
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/architecture/vector-search.md#sha256:c88c8c1c8d9f3ff889bbb31540cf1ef1746e7e9c022c7e864be6bf024282b812"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:d63afd36e3ebcd3145f77c24a6dd719be2216e458db791dbffe583dd5781c9c6"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:c8ec32d080168d72e02d3b5807408206493826cef0d4c6d1d0c735d5edc21897"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:4ebd4b395216cb325e559ffb99610c34b6976164c963fc14dc231832f5e5995d"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:ff9244ec2415296be250f87e099312c0fe1dfd58"
status: active
---

# `0.10.0` 벡터 검색 채택 기준

비벡터 수정의 `develop` 통합 뒤 전용 브랜치에서 구현. Markdown 정본과 FTS 유지.
동의된 비생성형 로컬 임베딩만 허용하고 기밀 조회·생성 승인은 분리.
범위별 검색·중단 복구·오래된 복사본 정리·증분 캐시 복원 구현.
채택은 기존 품질·용량·속도·운영체제 기준 통과가 필요.
원본 120문항과 독립 60문항의 결과를 따로 보고하며 새 시험으로 기존 실패를 지우지 않음.
안정판은 별도 명시 승인 전 금지.
