---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: en
counterpart: ../ko/v0-9-skill-suite-plan.md
title: "v0.9 Skill Suite Plan"
summary: "v0.9 finalizes host-native graph engineering, a unified Hive Wiki, regression-first cleanup, and bounded research without OMX/OMC or tmux dependencies."
tags: [graph-engineering, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop and Wiki plan"]
sources:
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:06938e887dc4992019718ea51ca0ec55f7bea4a56a647dd12409cd22c9375708"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:5801f3de1aedac7181d4e5eea44d0e3a94d0f45acb5d502b45b8e12145894f05"
links: [docs-wiki-architecture, orchestration-ownership, skill-routing]
reviewed_revision: "git:8414989a4f7822f8cbdf5e936d984150700825a4"
status: active
---

# v0.9 Skill Suite Plan

The approved v0.9 plan combines host-native subagent, goal, and hook capabilities
through `hive-loop-engineering`; keeps graph and evidence state in canonical
`.hive` Markdown; and unifies Wiki verbs through `hive-wiki`. Its utility suite
adds regression-first, behavior-preserving `ai-slop-cleaner` and terminal,
read-only `best-practice-research`. An explicit capability inventory governs
`adopt|merge|exclude` decisions. Hive adds no model runtime, scheduler, tmux,
OMX/OMC command or namespace dependency, automatic external-adapter priority,
Stop-hook continuation, or raw-session capture.
