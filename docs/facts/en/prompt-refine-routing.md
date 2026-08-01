---
schema_version: 1
pair_id: prompt-refine-routing
topic_slug: prompt-refine-routing
language: en
counterpart: ../ko/prompt-refine-routing.md
title: "Prompt Refine Approval Routing"
summary: "Materially ambiguous work enters refine-only and stops for exact user approval."
tags: [prompt, routing, skill]
aliases: ["Prompt approval gate"]
sources:
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:59129f4216306b3c095ab64574700135da0f289df4aab6554f0213e24c40c6f3"
  - "repo:docs/plans/active/prompt-refine-auto-routing.md#sha256:2c70a7ef894396d4bf1a3160c59d42f81e60c80905fe7c00d5f19f33de411b03"
links: [orchestration-ownership, skill-routing]
reviewed_revision: "git:507cdf98de2b0873b0e554fd1bc53810b11c7dc0"
status: active
---

# Prompt Refine Approval Routing

Hive will route explicit prompt authoring and materially ambiguous ordinary work
to `hive-prompt-refine` in `refine-only` mode. The refined prompt and digest enter
`awaiting-approval` with no project read, tool, write, memory capture, run creation,
or task execution. Only explicit `--run` or later approval bound to the exact digest
authorizes host-owned execution. Simple questions, editless questions, clear work,
and prompt-classifier hooks remain outside this route.
