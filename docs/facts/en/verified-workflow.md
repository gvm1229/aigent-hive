---
schema_version: 1
pair_id: verified-workflow
topic_slug: verified-workflow
language: en
counterpart: ../ko/verified-workflow.md
title: "Verified Workflow Skill"
summary: "0.10.0 renames ralph-loop to verified-workflow and routes complex natural-language continuation through evidence-gated execution graphs."
tags: [orchestration, skills, v0-10]
aliases: ["ralph-loop"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:1645eb2249265b75d27b0c65a709806f4999a0ec425e8e874336bcda084b702c"
  - "repo:docs/decisions/product-release-decisions.md#sha256:25bd2880270b2dd21bf09d5efe576f4164b8d02fadd8366f8649d8d50d38bded"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:db8825d8aa1d26905c55ba4a0c2892d8ef337551ff5a324ae69bb23d2ee56a93"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:a2518fa364c40efb4e676fe31b694562f73dd819"
status: active
---

# Verified Workflow Skill

`verified-workflow` replaces the `ralph-loop` public identity. Natural continuation selects it only
when at least two declared signals require dependency edges, intermediate evidence, bounded retry,
independent verification, steering, or exact recovery. Task length and a bare continue request do
not select it. The active host owns every task launch.
