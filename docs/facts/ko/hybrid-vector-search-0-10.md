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
  - "repo:docs/architecture/vector-search.md#sha256:4789dac545a49436d777c5bffa28b31cbb0e47a8bb34396363a214b9d2ebeeb8"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:08b5950d158c3e374752b625ba93a715d44c990d32c2ef39490bef2a1b9b084d"
  - "repo:docs/guides/vector-search.md#sha256:031e3db0bfd3dddf932a012bff98ed213dcb5a542f39921813c41acd109b5fb3"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:de7fb2f8aadab112f3b86da075fd972a4edcf999787eaf7585edcfe1a9fab4a9"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:32547d3a1332b6da5366bbeb9b95c968bb57cb83b115bbd563181ff676465847"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:d9cb7733237df4e5cc14824cb2df13ff75009776"
status: active
---

# `0.10.0` 벡터 검색 채택 기준

Markdown 정본·FTS 유지. 로컬 임베딩은 별도 동의, 기밀 조회·생성 승인은 분리.
의미 질문 두 평가 모두 58/60, 정확 질문 평균 역순위 0.975 유지. 번호 의미 조회 60개도 FTS 1위 보존.
Windows 전체 638.597초·증분 51.989초·전역/모음 조회 p95 3.067/2.276초·100DB SQL p95
98.876ms로 시간 기준 미달. 벡터 5만 개 일치·저장량 442.8MB 통과.
기능 검증 완료와 별개로 채택·test.5 대기. 작업 단위 모델 유지는 미승인 제안.
새 권한·바이트 검사 유지, 안정판은 명시 승인 전 금지.
