---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "Current Windows single-session and latched restart stops pass; full acceptance and install pending."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:361357e42576b8cbb6f01ee77a6eb0474cc6af4dcb68ddc6d18ede18208bd764"
  - "repo:crates/hive-cli/src/usage_monitor/desktop.ps1#sha256:4b6e8a788b28e12fc34dff7f07cdf03d98814b86b7235dec7e3ea42866deb7e9"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:4ef948e0b4201c4ac5a2864174c58f5d3ba07a3c54eaeafe75c075e4d870a59a"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:c45862bad77bd8cb0b9eb04691cf6161074509e7afeed1effb21cc57b03f532f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b6df0ad8e439ecfeb01d93e827e4d0f9b54ff8194d52149c41a3869f689ee6e0"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:d5a18bc4b139cd05ff96cd6c9497fe43a15b170c5eeb436523468e9115752eac"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:583b3083b9c547fcc385249fe6892c17224fe612"
status: active
---

# Session-Bound Reset-Only Control

0.12.1 iteration 1 stops sessions on observed reset or a 60s valid-observation gap. Starts: 15s; timeout: 5s; retry: 2s. Normal history subscriptions are absent; incident verification lasts at most 5s. App termination is forbidden. Shared incidents require exact acknowledgement and fresh usage; new chats and restarts cannot clear them. Work never resumes automatically. Windows Codex 26.1013.4894.0 single-session and latched restart stops pass with its window preserved. Full acceptance and installation remain pending; CLI 0.12.0 is installed and global monitoring is off. Threshold/zero-quota guards remain. Other OS, remote and cloud are unsupported; provider refill and billing cutoff are unproven.
