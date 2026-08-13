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
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:616b6850533a66d89c369cf1660987ca3760468a74054fc35c3871a98dda464d"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:e659d878b301c03648849ad0fd73bf59e31730fd"
status: active
---

# Model-Routed Custom Subagents

The `MRA-*` plan fixes exact Codex and Claude model/effort mappings for bounded
implementer and specialist roles. `hive agent recommend` now requires a protected,
externally signed host-model catalog, detached attestation, and trust root before it
prepares a decision. A forged signature or an incomplete exact mapping is rejected.
Fresh-host capability and lifecycle acceptance remain required before activation.
Manual and revised requests bind the exact prior decision digest, prior request, and scope.
