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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:bbe8cfa89ca1ff9f94c0820d77acce54beead1f572f888b762f07f866c8a00a4"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# Verified Workflow Skill

`verified-workflow` absorbs `ralph-loop` graph design and `iterative-execution` receipt and recovery
contracts. Natural continuation selects it only
when at least two declared signals require dependency edges, intermediate evidence, bounded retry,
independent verification, steering, or exact recovery. Task length and a bare continue request do
not select it. The active host owns every task launch. The read-only closure result exposes the
outer owner and a host-owned, non-spawning continuation envelope.
