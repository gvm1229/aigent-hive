---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "Iteration 1 uses reset or 60s observation-gap stops; desktop activation remains blocked."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:87fcbc7b005837f38639ceef43113c232ed5742298886caea92c009c9921d5cd"
  - "repo:crates/hive-cli/src/usage_monitor/desktop.ps1#sha256:ac6b2bafb909ef4afc591b16555346a15004b318fc6439de559e928f3d53ee99"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:91ab7f3d580cbc6f69a945072be09820c321c58ebcb247df033c5df88d85854b"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:c45862bad77bd8cb0b9eb04691cf6161074509e7afeed1effb21cc57b03f532f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:c51194775aaf8b9f4b0c34ffabdbbdd09768fb0758bdd54251d6f3d1a6f259b0"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:5035226eda596f7241ac9f730af31a78c72f61d2804b137849db1a572d7e1f47"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:7673c0ca72551d413d7f9e41f00d88c3f388b027"
status: active
---

# Session-Bound Reset-Only Control

The maintainer limited 0.12.1 iteration 1 to mid-turn stopping. Observed reset requests immediate interruption; 60s without valid usage also stops. Cadence: 15s starts, 5s timeout, 2s retry. Normal history subscriptions are absent; owners receive stop requests before bounded 5s verification. App termination is forbidden. Shared incidents survive opt-outs/restarts; exact acknowledgement plus fresh usage never resumes work. Thresholds/zero quota remain. Windows mocks pass; current desktop safety is unqualified, activation off and installed CLI 0.12.0. Earlier parent/subagent stops are partial evidence; app-exit causality remains unknown.
