---
schema_version: 1
pair_id: verified-execution-evidence
topic_slug: verified-execution-evidence
language: en
counterpart: ../ko/verified-execution-evidence.md
title: "Verified execution evidence"
summary: "Activation requires task-bound receipts; a node retry stop is not task completion."
tags: [orchestration, skills]
aliases: []
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:3a8450ff3e496f4e6bafc7b8d10cdd9fe38f15932b465d131a69ca0bdf9ef2f3"
  - "repo:.agents/directives/04-documentation-state.md#sha256:2626e090a19b45a88bc586c0292870dbf6136de40e3aa32359af2f617ead90a3"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:fc19bed8a17b8b8652c37ff518528ada2aec511e163b15c99af90235e6728a82"
links: [host-neutral-continuation, verified-workflow]
reviewed_revision: "git:5ea719a64f4403d1261feaff28d3f718d257638a"
status: active
---

# Verified execution evidence

The directive repair requires task-bound initialization and validation receipts before claiming
verified activation. A retry stop is not outer-task closure. Inspect closure readiness and current
criteria, not command success. Source work uses source policies; consumer run state cannot live at
the source root. Without a supported isolated run binding, continue under the source plan without
claiming verified execution. Instructions alone do not prove host-level final-response interception.
