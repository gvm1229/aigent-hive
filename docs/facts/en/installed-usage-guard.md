---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: en
counterpart: ../ko/installed-usage-guard.md
title: "Installed Guard Target Boundary"
summary: "The installed guard applies only to configured Hive projects and the Hive source workspace; non-Hive folders remain entirely inactive."
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:.github/workflows/ci.yml#sha256:bcba0d0f834f9e1e0dca81f465bb0337c5c4db83299c25d357f132f5a4cefd4d"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:06ce162411217587acab322726a197507bb179e318fc2b6157146e287ae3c15d"
  - "repo:docs/guides/installed-usage-guard.md#sha256:3224f7e04c9025cd788e14506295a723f1d87c97d59f9e629dcfe9bddcb1a302"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:39569b7a2a7c67f8ab19010db8c4df32da470f86"
status: active
---

# Installed Guard Target Boundary

The installed product is the sole usage-guard implementation. A configured Hive project uses
`max(global, project)` and may keep project-local session state. The Aigent Hive source workspace
uses the global threshold with user-root runtime state and no source `.hive/` files. A folder with
only its own `AGENTS.md`, or an empty folder, is non-Hive: no enforcement, threshold mutation,
session override, halt marker, or runtime file. Setup-free Hive Skills remain available there.
Source development uses one task-start preflight and no Python watcher, repeated tool-boundary
gate, or CI call to the removed source-guard test corpus.
