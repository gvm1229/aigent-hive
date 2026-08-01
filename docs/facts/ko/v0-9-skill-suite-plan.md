---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: ko
counterpart: ../en/v0-9-skill-suite-plan.md
title: "v0.9 Skill suite"
summary: "Host-native graph engineering·통합 Wiki·portable knowledge scan·RAG·회귀 우선 cleanup·bounded 연구의 v0.9 구현."
tags: [graph-engineering, rag, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop·Wiki 계획"]
sources:
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:06938e887dc4992019718ea51ca0ec55f7bea4a56a647dd12409cd22c9375708"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:ece47739f1d17b0d7ba604e5126fec55b445693335da10e54563b6cf2aa91224"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:2b7b1132b276dc59c0a00076d8aca13aebcb75eefb2dd66a3e1f9d51494fbba9"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:976150863fbb552b17b456b5bdaf4f6ce2780dcd7ed9af45ebcf565aae709e05"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:bcce739cb8ecafb0171f0cb7a9cba24da518383def08ab0dcb17086412814a7e"
  - "repo:docs/research/v0.9-omx-omc-capability-inventory.md#sha256:a8951fcd85427c238203e20d952a851f68e0ab50a18f1f5ab131ca101083061e"
links: [docs-wiki-architecture, global-knowledge-rag, knowledge-portability-scan, orchestration-ownership, skill-routing]
reviewed_revision: "git:d28c11908507cd0ae9f79ed0dfb4bcabf345ced2"
status: active
---

# v0.9 Skill suite

v0.9 구현: host-native graph engineering, canonical Markdown run 상태와 thin
`hive-wiki` router. `ai-slop-cleaner`, 읽기 전용 `best-practice-research`, checksummed
knowledge bundle, evidence-qualified `hive-knowledge-scan`, 기존 query Skill의 automatic
RAG 포함. 전체 역량의 `adopt|merge|exclude` 근거표 필수. Model runtime·scheduler·
tmux·OMX/OMC command·namespace·Stop continuation·raw session 수집 0건.
