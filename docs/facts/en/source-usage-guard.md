---
schema_version: 1
pair_id: source-usage-guard
topic_slug: source-usage-guard
language: en
counterpart: ../ko/source-usage-guard.md
title: "Installed Guard Target Boundary"
summary: "The installed guard applies only to configured Hive projects and the Hive source workspace; non-Hive folders remain entirely inactive."
tags: [guard, source, usage]
aliases: ["Source quota safeguard"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:06ce162411217587acab322726a197507bb179e318fc2b6157146e287ae3c15d"
  - "repo:docs/guides/source-usage-guard.md#sha256:c4c7f5f717627becc0636d1c7320eb227df844dc2ea5d837a79080c07c673197"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:907d4f3a0487bd7b0a8a0118b466eaf030064cc2"
status: active
---

# Installed Guard Target Boundary

The installed product is the sole usage-guard implementation. A configured Hive project uses
`max(global, project)` and may keep project-local session state. The Aigent Hive source workspace
uses the global threshold with user-root runtime state and no source `.hive/` files. A folder with
only its own `AGENTS.md`, or an empty folder, is non-Hive: no enforcement, threshold mutation,
session override, halt marker, or runtime file. Setup-free Hive Skills remain available there.
Source development uses one task-start preflight and no Python watcher or repeated tool-boundary
gate.
