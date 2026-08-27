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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:ccd2c8abeb632eab34d3b6772994e72be4960a1cf019538b57f53c706e9a51b4"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:f27203245c9c9020c77c8f29733d60f8783f3246b698635ceb01643c2ffa881d"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:571467bb776b86bed509a06cdb6744434b067993"
status: active
---

# `0.10.0` hybrid vector search gate

비벡터 수정의 `develop` 통합 뒤 `feature/0.10.0-vector-search`에서 벡터 구현 재개 승인.
이전 고유 1천 건 측정의 5만 건 환산값 2,711초는 600초 기준 실패의 과거 근거이며, 구현 완료 근거 아님.
기존 기준을 낮추지 않고 검색 품질·실제 전체 생성을 비교. FTS 유지, 승인형 비생성 로컬 색인 계산만 허용.
제공자 API·생성형 모델 프로세스 금지 유지. 기밀 검색과 색인 생성은 별도 작업 승인.
현재 벡터 제품 의존성 미추가.
