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
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:8dcf64600bf77f630d6f601027ee02a5adf1255a49c4c852ff6006a46f203817"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:6b2eb3faafe345678008fe225dd941026c8eab10911fa19530c2785c8b644f57"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:ffdfb476d4e21dafe5d4dc896fa272f7244d0fe1"
status: active
---

# Model-Routed Custom Subagents

The `MRA-*` plan fixes exact Codex and Claude model/effort mappings for bounded
implementer and specialist roles. A reserved user-scope Judge uses Sol High on Codex;
the Claude profile awaits lifecycle proof. User setup selects `explicit` or `implicit`.
Strict iterative, team, and multi-goal workflows always judge terminal acceptance,
never each tick. The agent returns a verdict, an external signer holds the Ed25519
private key, and Hive verifies bound receipts and quorum. The creator Skill cannot
override the reserved Judge.
