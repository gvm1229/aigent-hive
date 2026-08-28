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

# `0.10.0` 벡터 검색 채택 기준

비벡터 수정의 `develop` 통합 뒤 전용 브랜치에서 구현. Markdown 정본과 FTS 유지.
동의된 비생성형 로컬 임베딩만 허용하고 기밀 조회·생성 승인은 분리.
범위별 검색·복구·정리 구현. Windows 긴 경로·정본만 가져오기·소스 검색 통과.
MiniLM 점수 결합의 고정 질문 품질 통과.
공유 CPU 보조 실행기는 고유 5만 청크를 10분 안에 변환.
실제 100개 모음 CLI의 시간·용량·수치 동등성은 아직 미달. 운영체제 기준도 유지.
원본 120문항·독립 60문항 별도 보고. 새 시험으로 기존 실패 대체 금지.
임베딩만의 시간은 전체 검색 시간 근거에서 제외.
안정판은 별도 명시 승인 전 금지.
