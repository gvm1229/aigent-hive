---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: en
counterpart: ../ko/v0-9-skill-suite-plan.md
title: "v0.9 Skill Suite Plan"
summary: "v0.9 finalizes host-native graph engineering, unified Wiki and RAG, regression-first cleanup, and bounded research without OMX/OMC or tmux dependencies."
tags: [graph-engineering, rag, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop and Wiki plan"]
sources:
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:06938e887dc4992019718ea51ca0ec55f7bea4a56a647dd12409cd22c9375708"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:055496481dd5f0fa5ffcd92d6ddc6b456a01ce0db8edd998ccc3d2ae307f050e"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:4de98fc240cd60feb243a74ecbe4f46af79639f61d599a48f282cdc84b87ea3d"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:5801f3de1aedac7181d4e5eea44d0e3a94d0f45acb5d502b45b8e12145894f05"
links: [docs-wiki-architecture, global-knowledge-rag, orchestration-ownership, skill-routing]
reviewed_revision: "git:6e3eb11fb43b99971f73e1fed471ea6b34e8ba33"
status: active
---

# v0.9 Skill Suite Plan

The approved v0.9 plan combines host-native subagent, goal, and hook capabilities
through `hive-loop-engineering`; keeps graph and evidence state in canonical
`.hive` Markdown; and unifies Wiki verbs through `hive-wiki`. Its utility suite
adds regression-first, behavior-preserving `ai-slop-cleaner` and terminal,
read-only `best-practice-research`. An explicit capability inventory governs
`adopt|merge|exclude` decisions. Global knowledge RAG adds mandatory durable
memory writes and bounded retrieval before question routing. Hive adds no model
runtime, scheduler, tmux,
OMX/OMC command or namespace dependency, automatic external-adapter priority,
Stop-hook continuation, or raw-session capture.
