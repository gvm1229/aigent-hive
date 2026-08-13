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
  - "repo:crates/hive-cli/src/custom_agent_cli.rs#sha256:5726ce3e28f3198b267fc017cba94d53c4a8703efa74544e5499be7c9488d9dd"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9fa9e439ad15ea6a8b5ed7cf6d031595a8979b056dada55360cb32331d9e8355"
  - "repo:crates/hive-core/src/native_workflow.rs#sha256:246f845d21fe73c070abdfa4ffa78d28e829d84b3da498dcc1530355a54a0900"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:0beb197dff736c8569ce3d982fb1cd5bdd428dbec5d4581da7f2dad320613a29"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:9c9bdb1bfc49e06110fe3e1d0f931b03ab2c3b57"
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
