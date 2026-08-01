---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: en
counterpart: ../ko/v0-9-skill-suite-plan.md
title: "v0.9 Skill Suite"
summary: "v0.9 implements host-native graph engineering, unified Wiki, portable knowledge scanning, RAG, regression-first cleanup, and bounded research."
tags: [graph-engineering, rag, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop and Wiki plan"]
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

# v0.9 Skill Suite

The v0.9 suite implements host-native graph engineering, canonical Markdown run
state, and a thin `hive-wiki` router. It includes `ai-slop-cleaner`, read-only
`best-practice-research`, checksummed knowledge bundles, evidence-qualified
`hive-knowledge-scan`, and automatic RAG through the existing query Skill.
The complete capability inventory controls `adopt|merge|exclude`. Model runtime, scheduler,
tmux, OMX/OMC command or namespace dependencies, Stop continuation, and raw
session capture remain excluded.
