---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: en
counterpart: ../ko/model-routed-custom-subagents.md
title: "Model-Routed Custom Subagents"
summary: "Hive 0.9.0 plans exact-model custom-subagent routing for Codex and Claude with runtime attestation."
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor parity", "Task-appropriate model routing"]
sources:
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:fc30a26c372c5dd0881e5fcc36742820f49ae62ef4ee8fac410a60e8d8509fc0"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:b2b6be0e52d1b73542966859842381dcae1805353fd99173742411011fa731cf"
links: [orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:150c4ddd84eb80b0906da731aceb81c4f2d8e059"
status: active
---

# Model-Routed Custom Subagents

The `MRA-*` plan adds clean-room Sol Advisor parity for Codex and Claude in 0.9.0.
Each role fixes exact host model and effort, scope, permissions, trigger, and
ownership digest. Receipt mismatches are rejected. Built-ins cover routine and
complex implementation, fresh review, design, writing, research, and verification.
Purpose-first `hive-custom-subagent-create` recommends both host mappings and offers
accept, manual, or amend paths before registry integration. Activation requires
fresh-session proofs, explicit projection consent, and zero foreign-config overwrite.
