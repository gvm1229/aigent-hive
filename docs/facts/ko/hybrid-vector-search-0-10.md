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
  - "repo:docs/architecture/vector-search.md#sha256:ba354b18c8f8f4940d0387c002e904af43dc31d25aa9c72ef7541e9ba0f463bb"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/guides/vector-search.md#sha256:f39152f4aa591b4e8425ba7ebc3fe0274e4853c13eeb0605ebeac0fcfaa3674c"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:413f77fcba2a2ba20071aef4f8a0ac77582ed11e219c750dd312cbab189b3e9c"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:1e0c0ab8110ead87beab6f8c116c3dbc041451278c1a664445b75e83c51ec80f"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:74ede391133d6c92511774e8ee2957768ce70f05"
status: active
---

# `0.10.0` 벡터 검색 채택 기준

전용 벡터 브랜치에서도 Markdown 정본·FTS 유지. 로컬 임베딩은 별도 동의,
기밀 조회·생성 승인은 분리.
MiniLM의 원본·독립 의미 질문 정답 포함률 기준 통과. Windows 병렬 생성은 5만 청크·100개 모음
577.810초, 벡터 바이트 일치·누적 442.8MB로 통과. 이전 순차 증분 95.943초는 미달.
동일 CPU 종류 사용·모델 공유·게시 검사 병렬화 구현. 새 증분·전체 검색 실측 필요.
계산 구간만의 시간으로 전체 검색 속도 증명 금지. 새 권한·정본·바이트 검사 유지.
정확 조회의 FTS 순서 보호 뒤 원본 30개 순위 손실 0개·평균 역순위 0.975 유지.
안정판은 별도 명시 승인 전 금지.
