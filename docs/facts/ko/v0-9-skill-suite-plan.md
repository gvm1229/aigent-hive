---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: ko
counterpart: ../en/v0-9-skill-suite-plan.md
title: "v0.9 Skill suite 계획"
summary: "Host-native graph engineering·통합 Wiki·portable knowledge scan·RAG·회귀 우선 cleanup·bounded 연구의 v0.9 계획."
tags: [graph-engineering, rag, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop·Wiki 계획"]
sources:
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:06938e887dc4992019718ea51ca0ec55f7bea4a56a647dd12409cd22c9375708"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:ece47739f1d17b0d7ba604e5126fec55b445693335da10e54563b6cf2aa91224"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:0285b4850dfbb2651a2a8787c5d8c9b8c3b79cd3c3d1589c32106d2bb1847f43"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:437beec1bc0e37668162752ce8aa305ed73fc54a0ea27c5c7f3a4b160d9757f3"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:e82d39a3cb6687ecd4754d083fbb71ffdc39d7e53f03939cdab3190660cb8676"
links: [docs-wiki-architecture, global-knowledge-rag, knowledge-portability-scan, orchestration-ownership, skill-routing]
reviewed_revision: "git:4ef913efce07f4e86da98915c5ae5056dfac23e6"
status: active
---

# v0.9 Skill suite 계획

v0.9 suite: host-native graph engineering, canonical Markdown run 상태와 thin
`hive-wiki` router. `ai-slop-cleaner`, 읽기 전용 `best-practice-research`, checksummed
knowledge bundle, evidence-qualified `hive-knowledge-scan`, 기존 query Skill의 automatic
RAG 포함. 전체 역량의 `adopt|merge|exclude` 근거표 필수. Model runtime·scheduler·
tmux·OMX/OMC command·namespace·Stop continuation·raw session 수집 0건.
