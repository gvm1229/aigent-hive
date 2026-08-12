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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b5072a86240d87679c63a28568e1af75928367d037ddaa6af4cb9210bc4c2880"
  - "repo:docs/guides/source-usage-guard.md#sha256:c4c7f5f717627becc0636d1c7320eb227df844dc2ea5d837a79080c07c673197"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:ced55f4d0b18b259c9b43e0f9622b6d617a65737"
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
