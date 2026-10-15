---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "Windows control stays disabled until safe session-only interruption is qualified."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:5dd3088b00bc64fb53e3ad3be195b6b967e9b8e45433e4d915765b6626dd2d94"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:eea2f5a40cb2afa52e7acc6ed070b409d1f28f7b7bde3d545bce07baf9e8ab52"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b991ea10a2319c71ab62dea0c9ad3442dea2779d92e2d1bc215df1478077c6b4"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:a358f845a8f2a688af4b1dd5d081936d3673f960f1f6b37729cee01c75419a07"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:156c8cf863043b6e910c4b6bfa2dbb633382da49"
status: active
---

# Session-Bound Reset-Only Control

The 0.12.1 Windows controller is unqualified and disabled. Activation/run fail before app contact; direct PowerShell transport is blocked. App termination has no supported fallback. The active CLI was restored to 0.12.0; monitor tasks and automatic installation are off. Core logic keeps session-bound opt-outs, thresholds/zero quota, 15s starts, 5s timeouts, 2s retries, 30s blind stops and persistent incidents. Exact acknowledgement and fresh usage never resume work. Synthetic tests interrupted parent/subagents, but complete live acceptance failed. App-exit causality is unproven. Safe session-only stopping, visible status and installation remain unqualified.
