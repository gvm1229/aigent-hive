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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:fe327177fca73ccbdb3267a1cfca7b579b984e8bd3a24e74457a7d062020f2ec"
  - "repo:docs/decisions/product-release-decisions.md#sha256:59e330c3bd0a5a8133e00c447c99db44e30274dbf92770b662d3cf4c14b50e0f"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:c9399a4ad9e99389eb84aa5da5dbbae38cf20d5bca2f6c239322f2c81ad48d71"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:26e5fd299f961d79c6b8237c212b4b07e9e99770"
status: active
---

# Verified Workflow Skill

`verified-workflow` absorbs `ralph-loop` graph design and `iterative-execution` receipt and recovery
contracts. Natural continuation selects it only
when at least two declared signals require dependency edges, intermediate evidence, bounded retry,
independent verification, steering, or exact recovery. Task length and a bare continue request do
not select it. The active host owns every task launch.
