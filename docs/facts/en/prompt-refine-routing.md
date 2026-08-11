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
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:da1f8b5e6323b945a0a85740f32a8e4332cb3c9e9d446c6a1d5acc9846653454"
  - "repo:docs/plans/active/prompt-refine-auto-routing.md#sha256:a56c022be4e24ac6e7acf402e186d1ddbe4a1a39bc4d2c0eb16104e472b3108a"
links: [orchestration-ownership, skill-routing]
reviewed_revision: "git:bf7c1d3e36cd94e8ee5f2a68d9f8ca5c4c9f9c87"
status: active
---

# Prompt Refine Approval Routing

Hive will route explicit prompt authoring and materially ambiguous ordinary work
to `prompt-refine` in `refine-only` mode. The refined prompt and digest enter
`awaiting-approval` with no project read, tool, write, memory capture, run creation,
or task execution. Only explicit `--run` or later approval bound to the exact digest
authorizes host-owned execution. Simple questions, editless questions, clear work,
and prompt-classifier hooks remain outside this route.
