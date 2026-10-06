---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "Explicit reset-only opt-out preserves usage threshold protection and pending reset acknowledgement."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:41e5712b85c91de0f0df5505e999b245e60a530d5764f8cf4a35c285b8c4f730"
  - "repo:docs/guides/installed-usage-guard.md#sha256:916836c260db60424f19f6540ee5855e490e896f3681b658b07c8d1c063ddc05"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:ad412a7b2aa6bba927f5f841e0e5161f4fb60488"
status: active
---

# Session-Bound Reset-Only Control

Reset detection is enabled by default. Increased usage blocks new automatic work at the next check until the user acknowledges the reset and a fresh check allows continuation. This is not continuous monitoring or interruption of running work. The 0.11.0 disable-reset-guard and enable-reset-guard actions use exact host/session/process binding. Disabling reset detection requires confirmation, preserves threshold and unknown-usage blocks, and cannot clear a pending reset. Both actions require fresh enforce; new bindings inherit no opt-out. Windows CLI regressions verify these boundaries.
