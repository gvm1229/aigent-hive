---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: en
counterpart: ../ko/v0-9-skill-suite-plan.md
title: "v0.9 Skill Suite Plan"
summary: "v0.9 finalizes host-native graph engineering, unified Wiki, portable knowledge scanning, RAG, regression-first cleanup, and bounded research."
tags: [graph-engineering, rag, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop and Wiki plan"]
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

# v0.9 Skill Suite Plan

The v0.9 suite combines host-native graph engineering, canonical Markdown run
state, and a thin `hive-wiki` router. It adds `ai-slop-cleaner`, read-only
`best-practice-research`, checksummed knowledge bundles, evidence-qualified
`hive-knowledge-scan`, and automatic RAG through the existing query Skill.
Capability inventory controls `adopt|merge|exclude`. Model runtime, scheduler,
tmux, OMX/OMC command or namespace dependencies, Stop continuation, and raw
session capture remain excluded.
