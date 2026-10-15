---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "Session-only interruption preserves the app, visible reasons, and exact acknowledgement boundaries."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:7e20a151a1ed78c1aebbaa5c55856ef67dfbdbb765818ced004b46daf706330a"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:eea2f5a40cb2afa52e7acc6ed070b409d1f28f7b7bde3d545bce07baf9e8ab52"
  - "repo:docs/guides/installed-usage-guard.md#sha256:dc9e92ed1cb85336da05c2f030e20efcdebf15b60751b90777a2307ec8cefef4"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:517fbb1c2f45fca0bdcacb6fa68b3e85ba538a0293dc50fdd8311ec691038d6e"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:8f2d7121058024bb4118ae036c6878cc7eae75de"
status: active
---

# Session-Bound Reset-Only Control

Reset-only opt-outs stay session-bound and keep threshold/zero-quota checks. Windows 0.12.1 uses the qualified desktop user-stop path, keeps the app open and latches incidents across chats/restarts. Usage starts every 15s, with 5s limits, 2s retries and a 30s blind stop. Exact acknowledgement and fresh usage must release the controller; no automatic resume. A synthetic reset interrupted the parent and two subagents while the app survived. Full acceptance failed on repeated historical-chat checks; corrected verification remains pending. The window separates test/history, reasons and deadline failures. Installation acceptance, billing cutoff and overshoot bounds remain unproved.
