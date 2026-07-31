---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: en
counterpart: ../ko/v0-9-skill-suite-plan.md
title: "v0.9 Skill Suite Plan"
summary: "v0.9 plans host-native graph engineering, a unified Hive Wiki, and two reviewed utility Skills without a Hive scheduler."
tags: [graph-engineering, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop and Wiki plan"]
sources:
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:42306073c01e63e8728ba01b7ed2642598bea7e54cc418b07487a66c86a914c5"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:e1dcf45d5a612d71014159b001de0c0de2188b37cf539ef009ab84c2dd1b67b0"
links: [docs-wiki-architecture, orchestration-ownership, skill-routing]
reviewed_revision: "git:be5253bcbd0d9818333e5702d0ef9ce438ee4d62"
status: active
---

# v0.9 Skill Suite Plan

The v0.9 plan combines host-native subagent, goal, and hook capabilities through
`hive-loop-engineering`; keeps graph and evidence state in canonical `.hive`
Markdown; unifies Wiki verbs through `hive-wiki`; and adds `ai-slop-cleaner` and
`best-practice-research`. Hive adds no model runtime, scheduler, tmux dependency,
Stop-hook continuation, OMX namespace, or raw-session capture.
