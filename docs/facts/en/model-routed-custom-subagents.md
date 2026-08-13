---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: en
counterpart: ../ko/model-routed-custom-subagents.md
title: "Model-Routed Custom Subagents"
summary: "Hive 0.9.3 verifies signed host-model catalog evidence before preparing a custom-agent recommendation."
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor parity", "Task-appropriate model routing"]
sources:
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:42506fc775e4a456f724c73fc71a2fb1fc80c12967606accb909de3ef323c888"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:9bfb375de4806e6ca2659ec17ac1062676cafb0a601c449570a230cbbc2dc3ca"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:1b7b2de509823ca3c66ac6f6c7a0ba5ab84a071f"
status: active
---

# Model-Routed Custom Subagents

The `MRA-*` plan fixes exact Codex and Claude model/effort mappings for bounded
implementer and specialist roles. `hive agent recommend` now requires a protected,
externally signed host-model catalog, detached attestation, and trust root before it
prepares a decision. A forged signature or an incomplete exact mapping is rejected.
Fresh-host capability and lifecycle acceptance remain required before activation.
