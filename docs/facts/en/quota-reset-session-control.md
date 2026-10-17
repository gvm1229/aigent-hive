---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "Current Windows parent, shell and reasoning stops are observed; full confirmation and install pending."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:361357e42576b8cbb6f01ee77a6eb0474cc6af4dcb68ddc6d18ede18208bd764"
  - "repo:crates/hive-cli/src/usage_monitor/desktop.ps1#sha256:637e61a3dfad40f2b910feda206280cb8f2b23104155f5e39638bd6415caf916"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:4ef948e0b4201c4ac5a2864174c58f5d3ba07a3c54eaeafe75c075e4d870a59a"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:c45862bad77bd8cb0b9eb04691cf6161074509e7afeed1effb21cc57b03f532f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b6df0ad8e439ecfeb01d93e827e4d0f9b54ff8194d52149c41a3869f689ee6e0"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:89316ee7645ac9df0d5a93a480ade38314ac3970b11f3ab0fc27b029bef87527"
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:e4bb6407c618423280f980006c64aa6c6c2eb17f52d2fee44df5abe8552af36b"
  - "repo:tests/results/session-stop-partial-windows-0.12.1.json#sha256:1f1d27dae9ef95f345c39fa018dce9f01b752cd765d24de3609b1ac7280a4071"
links: [automatic-dispatch-guard, installed-usage-guard, usage-stop-notice]
reviewed_revision: "git:ee69dbe35105053232d9029e50007519f6052fbe"
status: active
---

# Session-Bound Reset-Only Control

0.12.1 iteration 1 stops sessions on observed reset or a 60s valid-observation gap. Starts: 15s; timeout: 5s; retry: 2s. Normal history subscriptions are absent; incident verification lasts at most 5s. App termination is forbidden. Shared incidents require exact acknowledgement and fresh usage; new chats and restarts cannot clear them. Work never resumes automatically. Windows Codex 26.1013.4894.0 parent, shell and tool-free reasoning stops are observed with its window preserved. Full confirmation is being corrected; installation remains pending; CLI 0.12.0 is installed and global monitoring is off. Threshold/zero-quota guards remain. Other OS, remote and cloud are unsupported; provider refill and billing cutoff are unproven.
