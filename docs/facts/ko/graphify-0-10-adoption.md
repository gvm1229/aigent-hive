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
  - "repo:docs/plans/active/knowledge-relationship-graph-0.10.0.md#sha256:cb76dcf1ca263a1fb5f0f7e0c030decd04c4dd65207980cb0698f1451848109c"
  - "repo:docs/plans/backlog/graphify-knowledge-graph.md#sha256:6ab392f6613412116a8fc24ad447236f319ceea7dee257ace138a300fc3cf960"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:docs/research/graphify-0.10-feasibility.md#sha256:a86ab9852f0fbd1a1737c97fcd236cf140fdc194e1f00d0e213e213a6f2fe600"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
status: active
---

# Graphify 0.10 도입 판정

- Graphify `0.9.47` 확인: 반복 전체 구조 graph 생성과 작은 Windows 조회 성능 통과
- 하드 게이트 실패: 같은 자료의 증분 갱신·전체 재생성 결과 불일치, 단일 upstream
  global graph의 collection 공개 범위 격리 부재
- 전면 지식 graph: `0.10.0` 범위 제외
- 승인 범위: Hive-native Markdown 관계 graph와 Graphify full-rebuild code-only adapter
- 구현: Exact consent·세 target wheel lock·code-only receipt·grounded locator·atomic activation·native fallback
- 남은 범위: 세 운영체제 공개 시험 수용
