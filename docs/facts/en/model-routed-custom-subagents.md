---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: en
counterpart: ../ko/model-routed-custom-subagents.md
title: "Model-Routed Custom Subagents"
summary: "Hive custom-agent activation requires exact runtime attestation independently of native host delegation support."
tags: [antigravity, claude, codex, model-routing, subagent, v0-10, v0-9]
aliases: ["Sol Advisor parity", "Task-appropriate model routing"]
sources:
  - "repo:crates/hive-cli/src/custom_agent_cli.rs#sha256:41e7e1bded6372419575a428f2ab1bdda9f163a294eb2f2b6c275f118781207c"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a50a29d628ce2e15e20b21fd74964ae96c493b259ffeabe0eade38cde54991aa"
  - "repo:crates/hive-core/src/native_workflow.rs#sha256:246f845d21fe73c070abdfa4ffa78d28e829d84b3da498dcc1530355a54a0900"
  - "repo:docs/archive/plans/foundations/model-routed-custom-subagents.md#sha256:9fe4b79c4f4e0be1706600e06b74ab93ee8bbce01e767a38790bbf8bdd21b251"
  - "repo:docs/research/host-work-delegation-2026-08-20.md#sha256:00e8c2821082ececec3cbef81538030fc9487a8ba0903f1ee1fb378d73aa6c74"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Model-Routed Custom Subagents

`hive agent recommend` requires a signed host model catalog, detached attestation, and trust root.
Forged signatures, incomplete exact mappings, and unsupported lifecycle capability fail before
activation. Both Judge policies allow strict terminal acceptance. Material risk requires
`implicit`. The 2026-08-20 review confirmed native delegation in Codex, Claude Code, and
Antigravity. Hive automatic activation remains blocked because no host supplies an externally
verifiable receipt that binds the exact role, model, effort, and definition digest.
