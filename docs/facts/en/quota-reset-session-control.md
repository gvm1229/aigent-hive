---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "Reset-only session controls and the separate Windows app interruption monitor preserve distinct acknowledgement boundaries."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:12d7f863da599795e3a7c1f6292bffdb68825bae9d12aaf77f453ee6bcf5cbda"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:059c527509e30d2895f9af3a056af6669eda4061dab016f76b6ec17213a38c03"
  - "repo:docs/guides/installed-usage-guard.md#sha256:f1a23a8779e4272fd852313360653f4078bcabd1290c4bebda9c8b0589d077fc"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:eb62d9db75f946f948b4dada89fa141654e80135"
status: active
---

# Session-Bound Reset-Only Control

Reset-only opt-outs require host/session/process binding and confirmation; they preserve thresholds, unknown-usage blocks and pending resets. Fresh enforce follows each change. Since 0.12.1, the separate Windows Codex monitor compares shared observations every 15 seconds, latches resets across chats/restarts, and requests app-wide emergency termination. A 30-second observation gap also halts for quota protection with an external notification. Exact incident acknowledgement and fresh usage never resume work automatically. Windows state and process-fixture tests verify these paths; actual app acceptance and installation remain separate. Origin: requested mid-turn halt after quota refill.
