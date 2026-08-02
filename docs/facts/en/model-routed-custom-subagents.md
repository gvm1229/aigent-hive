---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: en
counterpart: ../ko/model-routed-custom-subagents.md
title: "Model-Routed Custom Subagents"
summary: "Hive 0.9.0 plans exact-model Codex and Claude roles with a configurable authenticated Judge."
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor parity", "Task-appropriate model routing"]
sources:
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:cc7e79da6c27052fb9dc256a47e057deab617bbb66a567da8455d9135d6407b8"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:a2779d3f1ebab829c48214fd4486f9505e7207b6de1cdfe3c6af56c9121534ce"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:4e750ce659c953d7d71ab6e9536c29968ab1f028"
status: active
---

# Model-Routed Custom Subagents

The `MRA-*` plan fixes exact Codex and Claude model/effort mappings for bounded
implementer and specialist roles. A reserved user-scope Judge uses Sol Max on Codex;
the Claude profile awaits lifecycle proof. User setup selects `explicit` or `implicit`.
Strict iterative, team, and multi-goal workflows always judge terminal acceptance,
never each tick. The agent returns a verdict, an external signer holds the Ed25519
private key, and Hive verifies bound receipts and quorum. The creator Skill cannot
override the reserved Judge.
