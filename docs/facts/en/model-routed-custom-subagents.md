---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: en
counterpart: ../ko/model-routed-custom-subagents.md
title: "Model-Routed Custom Subagents"
summary: "Hive 0.9.3 custom-agent activation requires every host lifecycle capability, and Judge invocation persists only explicit or implicit policy."
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor parity", "Task-appropriate model routing"]
sources:
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:42506fc775e4a456f724c73fc71a2fb1fc80c12967606accb909de3ef323c888"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:c713d13630d17bb216771b6e18b9efb0f4fb38bb0f9de312b3220e5b48ae1282"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:271ad81dbf3389ba8932c5617738cf5f9022c5c9"
status: active
---

# Model-Routed Custom Subagents

`hive agent recommend` requires a protected, externally signed host-model catalog,
detached attestation, and trust root. Incomplete mappings and forged signatures fail closed.
Activation requires every supported host lifecycle capability. Manual and revised requests bind
the exact prior decision digest, request, and scope. User setup persists only `explicit` or
`implicit` Judge invocation, with the same public default and enum.
Hostile regressions preserve foreign bytes when a receipt is absent, a role collides, or a
projection path crosses a symlink.
