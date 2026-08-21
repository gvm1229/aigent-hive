---
schema_version: 1
pair_id: graphify-0-10-adoption
topic_slug: graphify-0-10-adoption
language: ko
counterpart: ../en/graphify-0-10-adoption.md
title: "Graphify 0.10 도입 판정"
summary: "Graphify 전면 지식 graph 제외와 code-only 선택형 adapter 권장안의 0.10.0 범위 승인 대기."
tags: [graphify, knowledge, security, v0-10]
aliases: ["Graphify 도입", "knowledge graph 판정"]
sources:
  - "repo:docs/plans/backlog/graphify-knowledge-graph.md#sha256:1558ec29827e73f622ce5f978f07e8f800c12600b1efbe75275e2ef072096431"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:63ec71461c610b4ccab8e186d8337f504b28ca5cd1a25dcd793872e7960bb427"
  - "repo:docs/research/graphify-0.10-feasibility.md#sha256:6bc52a7a6fa89601c5b20d851cb721e6b0f5d0e59b51b6d18963baaa69b6930e"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:daa32013cd5c9f506551532d0a5692d5644aeeaf"
status: active
---

# Graphify 0.10 도입 판정

- Graphify `0.9.47` 확인: 반복 전체 구조 graph 생성과 작은 Windows 조회 성능 통과
- 하드 게이트 실패: 같은 자료의 증분 갱신·전체 재생성 결과 불일치, 단일 upstream
  global graph의 collection 공개 범위 격리 부재
- 전면 지식 graph: `0.10.0` 범위 제외
- 권장 후보: Hive-native Markdown 관계 graph와 Graphify full-rebuild code-only adapter
- 현재 상태: `SCP10-001` 유지보수자 범위 승인 대기, 제품 통합 `0건`
