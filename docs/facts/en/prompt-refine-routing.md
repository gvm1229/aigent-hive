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
  - "repo:docs/archive/plans/foundations/prompt-refine-auto-routing.md#sha256:a56c022be4e24ac6e7acf402e186d1ddbe4a1a39bc4d2c0eb16104e472b3108a"
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:9091a6094f11be32f27108944ec98adbd0dc425afb6faa26ba8cf616f18d8896"
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
