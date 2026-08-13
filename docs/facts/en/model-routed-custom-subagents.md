---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: en
counterpart: ../ko/model-routed-custom-subagents.md
title: "Model-Routed Custom Subagents"
summary: "Hive 0.9.3 custom-agent activation requires every host lifecycle capability and a closed Judge invocation policy."
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor parity", "Task-appropriate model routing"]
sources:
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cffd6c491ffd17dccefa84edb172bbfe64ae925f2fe9cf7c6efd07e6a896a9fd"
  - "repo:crates/hive-core/src/native_workflow.rs#sha256:246f845d21fe73c070abdfa4ffa78d28e829d84b3da498dcc1530355a54a0900"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:3c19d66b868d0b07f03d7d7eda62c0cd4c3d2db46920e9cfc65f8c5b0967f165"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:da8ff786068c1cf28b0e40862494767ddeffe9c0"
status: active
---

# Model-Routed Custom Subagents

`hive agent recommend` requires a protected signed host-model catalog, detached attestation, and
trust root. Forged signatures, incomplete mappings, and unsupported lifecycle capability fail
before activation. Manual and revised requests bind the exact prior digest, request, and scope.
Strict terminal acceptance is permitted in both Judge policies; material risk requires `implicit`.
Simple, read-only, format-only, scheduler, heartbeat, retry, deterministic-failure, and
unsupported-host routes are always rejected. Hostile regressions preserve foreign bytes when a
receipt is absent, a role collides, or a projection path crosses a symlink.
