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
  - "repo:docs/architecture/vector-search.md#sha256:91883e7c324dab8bc8beb7cfe1f39a7cde6ddef9024dbd873b1b7adb52e0dd54"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/guides/vector-search.md#sha256:e24d70629bf128f5dddfbe947e52841b6d71bdcf241d65527b420acf6bc93e54"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:8c1a2188796d37696736b87097f07cd54b3a311591650770af32a2a795e58634"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:a014ced35c5a1d072fe5e8b3866b5531241b1a70c2d8bb0b454c92ddee8c214b"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:ddb91859aae17ee52c79ca2b14fdaebb5f2876dd"
status: active
---

# `0.10.0` 벡터 검색 채택 기준

전용 벡터 브랜치에서도 Markdown 정본·FTS 유지. 로컬 임베딩은 별도 동의,
기밀 조회·생성 승인은 분리.
MiniLM의 원본·독립 질문 품질 통과. Windows 5만 청크·100개 모음의
순차 생성 932.943초·증분 95.943초로 시간 미달, 벡터 보존·누적 442.8MB 통과.
동일 CPU 종류 사용·모델 공유 병렬 처리·게시 검사 병렬화 구현, 최종 실측 필요.
계산 구간만의 시간으로 전체 검색 속도 증명 금지. 새 권한·정본·바이트 검사 유지.
안정판은 별도 명시 승인 전 금지.
