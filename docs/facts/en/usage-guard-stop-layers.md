---
schema_version: 1
pair_id: usage-guard-stop-layers
topic_slug: usage-guard-stop-layers
language: en
counterpart: ../ko/usage-guard-stop-layers.md
title: "Usage Guard Stop Layers"
summary: "Historical cooperative gates and native live interruption have different guarantees."
tags: [guard, history, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:79a51f9f11e9bc01cbc97bb6d0943068f325dcac473cdf3070d9e2aee413846a"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:798f0e43b17f3446547899b0355cca2b177779889b1245ce4cec7dd0bfcc26de"
  - "repo:crates/hive-cli/src/usage_monitor/threshold.rs#sha256:9f00da30cddf8bbc0083c2e90786e3061fe50947cf41e183c4c8ee9bd91af5c1"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:0c6cf716cd863a1a15a1cb842be38302e1a8a803b068bcc2ea4e0e0b6d2316c1"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:e7afab494cd37c690cf68a4bd49d90a3251e4396"
status: active
---

# Usage Guard Stop Layers

The 0.7/0.8 Skills and 0.9 guard block new automatic dispatch. The source-only 15s watcher wrote halt.json for the next gate; it did not interrupt inference and was removed in August 2026. Opt-in 0.13.0 reuses sensing/reset data with independent native Codex user-stop for reset or 60s gaps. Development 0.13.1 shares the floor policy and owner-scoped manual-resume receipts. Windows long reasoning and same-floor resume are verified. Shell chat cancellation did not terminate the test child within 5s; aggregate acceptance remains incomplete. Earlier gates and Windows proof do not establish historical inference interruption or support on other hosts.
