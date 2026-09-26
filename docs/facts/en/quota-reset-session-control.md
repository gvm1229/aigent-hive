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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:c94975f1e11052ebf9c04e00066fe121229eea186945c056164d3a33c609df87"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:72bd93df896956e10c30013db3bc9232b7a4a3ca"
status: active
---

# Session-Bound Reset-Only Control

Reset detection is enabled by default. Increased usage blocks new automatic work at the next check until the user acknowledges the reset and a fresh check allows continuation. This is not continuous monitoring or interruption of running work. The 0.11.0 disable-reset-guard and enable-reset-guard actions use exact host/session/process binding. Disabling reset detection requires confirmation, preserves threshold and unknown-usage blocks, and cannot clear a pending reset. Both actions require fresh enforce; new bindings inherit no opt-out. Windows CLI regressions verify these boundaries.
