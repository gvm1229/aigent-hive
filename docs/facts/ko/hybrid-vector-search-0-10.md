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
  - "repo:docs/architecture/vector-search.md#sha256:7200f812a0d660d5740ccf0cd656095e0f266de8f120a1ba4738010a6f940d2a"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/guides/vector-search.md#sha256:0c3fe94600b5ec85ff34dcc0eec814a9d4fabca1772ffc5021c5d97478dbfd0d"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:413f77fcba2a2ba20071aef4f8a0ac77582ed11e219c750dd312cbab189b3e9c"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/evidence/vector-requalification-windows-2026-08-24.json#sha256:df1a2e0bf1001236cef266653309154bb99676837be86a2beba25e8dff16b178"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
  - "repo:docs/research/vector-product-integration-2026-08-28.md#sha256:6c7e69005d9e9a2883414a7c567ff48c9ab60bc6489409e4e5be7d1d52f0f8b0"
  - "repo:docs/research/vector-requalification-0.10-2026-08-24.md#sha256:8e7a9a70df255694b10bc88b9dadb40619ad36f74d20902b06dea1db556f595e"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:c6f1663011110ebe7a09f655e2e2f663083be8af"
status: active
---

# `0.10.0` 벡터 검색 채택 기준

Markdown 정본·FTS 유지. 로컬 임베딩은 별도 동의, 기밀 조회·생성 승인은 분리.
MiniLM 원본·독립 의미 질문 기준 통과. 최신 Windows 5만 청크·100모음 재생성은
638.597초로 시간 미달. 벡터 바이트 일치·누적 442.8MB 통과, 최신 증분 51.989초는 미달.
같은 CPU 종류·모델 공유·정본 병렬 검사·EOF 보존 버퍼 적용.
최종 검색 실측 필요. 계산 구간만으로 전체 검색 속도 증명 금지.
새 권한·정본·바이트 검사 유지. FTS 순서 보호 뒤 정확 30개 순위 손실 0개·평균 역순위 0.975.
안정판은 명시 승인 전 금지.
