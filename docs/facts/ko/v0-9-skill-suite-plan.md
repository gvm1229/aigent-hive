---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: ko
counterpart: ../en/v0-9-skill-suite-plan.md
title: "v0.9 Skill suite 계획"
summary: "OMX/OMC·tmux 의존 없이 host-native graph engineering·통합 Wiki·RAG·회귀 우선 cleanup·bounded 연구를 확정한 v0.9 계획."
tags: [graph-engineering, rag, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop·Wiki 계획"]
sources:
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:06938e887dc4992019718ea51ca0ec55f7bea4a56a647dd12409cd22c9375708"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:055496481dd5f0fa5ffcd92d6ddc6b456a01ce0db8edd998ccc3d2ae307f050e"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:4de98fc240cd60feb243a74ecbe4f46af79639f61d599a48f282cdc84b87ea3d"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:5801f3de1aedac7181d4e5eea44d0e3a94d0f45acb5d502b45b8e12145894f05"
links: [docs-wiki-architecture, global-knowledge-rag, orchestration-ownership, skill-routing]
reviewed_revision: "git:6e3eb11fb43b99971f73e1fed471ea6b34e8ba33"
status: active
---

# v0.9 Skill suite 계획

v0.9 최종 계획: host-native subagent·goal·hook을 조합하는
`hive-loop-engineering`, canonical `.hive` Markdown graph·evidence 상태,
Wiki 동사를 통합하는 `hive-wiki`, 회귀 시험 우선·동작 보존형
`ai-slop-cleaner`, 읽기 전용·bounded `best-practice-research`. 전체 역량의
`adopt|merge|exclude` 근거표 필수. 모든 질문 전 bounded retrieval과 durable memory
mandatory write의 전역 knowledge RAG 포함. Model runtime·scheduler·tmux·OMX/OMC
command·namespace·자동 외부 adapter 우선권·Stop-hook continuation·raw session 수집 0건.
