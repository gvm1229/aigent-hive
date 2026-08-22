---
schema_version: 1
pair_id: graphify-0-10-adoption
topic_slug: graphify-0-10-adoption
language: ko
counterpart: ../en/graphify-0-10-adoption.md
title: "Graphify 0.10 도입 판정"
summary: "0.10.0의 Hive-native Markdown 관계 graph·선택형 Graphify full-rebuild code-only adapter 승인과 전면 Graphify 지식 graph 제외."
tags: [graphify, knowledge, security, v0-10]
aliases: ["Graphify 도입", "knowledge graph 판정"]
sources:
  - "repo:docs/plans/active/knowledge-relationship-graph-0.10.0.md#sha256:375535cd3b4e94ec68b9d61ebd11db8a4c5cc07e5480717a48f80f58424ea7d7"
  - "repo:docs/plans/backlog/graphify-knowledge-graph.md#sha256:6ab392f6613412116a8fc24ad447236f319ceea7dee257ace138a300fc3cf960"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:docs/research/graphify-0.10-feasibility.md#sha256:6bc52a7a6fa89601c5b20d851cb721e6b0f5d0e59b51b6d18963baaa69b6930e"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:c78c7993807302d4e66246aacc7732c4848eb4b0"
status: active
---

# Graphify 0.10 도입 판정

- Graphify `0.9.47` 확인: 반복 전체 구조 graph 생성과 작은 Windows 조회 성능 통과
- 하드 게이트 실패: 같은 자료의 증분 갱신·전체 재생성 결과 불일치, 단일 upstream
  global graph의 collection 공개 범위 격리 부재
- 전면 지식 graph: `0.10.0` 범위 제외
- 승인 범위: Hive-native Markdown 관계 graph와 Graphify full-rebuild code-only adapter
- 현재 상태: native graph schema·직접 Markdown edge 추출 구현, 수용 대기
