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
  - "repo:.agents/directives/01-behavior.md#sha256:49c79d8137a1e1d3cdcb06b60fb7e2708e2cf0ae04018a6c9c866ecb0a65a333"
  - "repo:.agents/directives/references/run-closure.md#sha256:e55fe262c9a36e7a1c0f372941cc7334f5c4f78d888872add82635c0015865a2"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:b540e5ca68afee2e3947932e9b21bef1c5707cbde322d7c89cd287965609d5cf"
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
