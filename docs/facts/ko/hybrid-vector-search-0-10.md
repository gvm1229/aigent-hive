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
  - "repo:docs/architecture/vector-search.md#sha256:fd32cbf709f0169af197740a45e00d848b5c46bff88b62c4540ff5fbefe0b695"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/guides/vector-search.md#sha256:e24d70629bf128f5dddfbe947e52841b6d71bdcf241d65527b420acf6bc93e54"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:8c1a2188796d37696736b87097f07cd54b3a311591650770af32a2a795e58634"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:b6cd492d12490ae50de67a374d015be34e7755b2df97d573c3af66c4f79c411d"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:c264d0e315249dc2f95a58b2c4ab02375b0acad4"
status: active
---

# `0.10.0` 벡터 검색 채택 기준

전용 벡터 브랜치에서도 Markdown 정본·FTS 유지. 로컬 임베딩은 별도 동의,
기밀 조회·생성 승인은 분리.
MiniLM의 원본·독립 질문 품질 통과. Windows P 코어 실험의 5만 청크·100개 모음
생성 532.66초, 이전 세대 포함 372.2MB. P/E 실행에 따른 수치 차이 재현 후
허용된 한 종류의 CPU만 쓰도록 보강. 증분 잠금 충돌·전체 검색 시간은 여전히 미달.
계산 구간만의 시간을 전체 검색 속도로 보고 금지. 최종 구성 재검증 필요.
안정판은 별도 명시 승인 전 금지.
공유 모음 묶음 갱신과 단일 검색의 SQLite 연결 재사용에서도 새 권한·바이트 검사 유지.
