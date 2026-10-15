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
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:33b5733343ccefcf6d212a98729884f1952e1ab87a37a5715ed20f4197dcf39c"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:eea2f5a40cb2afa52e7acc6ed070b409d1f28f7b7bde3d545bce07baf9e8ab52"
  - "repo:docs/guides/installed-usage-guard.md#sha256:dc9e92ed1cb85336da05c2f030e20efcdebf15b60751b90777a2307ec8cefef4"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:173c8739c9e593238cfc0b55cbbe52b0033d53dc"
status: active
---

# Session-Bound Reset-Only Control

Reset-only opt-outs preserve thresholds and zero-quota checks. The 0.12.1 Windows monitor uses the qualified desktop coordination pipe and user-stop path; app termination is unavailable. It latches incidents across chats/restarts, compares fresh usage every 15 seconds, bounds queries at five seconds, retries after two, and halts after a 30-second valid-observation gap. Exact acknowledgement plus fresh usage waits for controller release; it never resumes work. The status window distinguishes interruption, app survival and failure. Source regressions and current-desktop read qualification do not replace live interruption, visible-reason and installation acceptance. No server billing cutoff or overshoot bound is proven.
