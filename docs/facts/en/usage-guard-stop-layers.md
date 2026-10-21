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
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:09a27cf643e59088c6966d4a10e659069662860c8646db86b195f5bd1d13bafb"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:798f0e43b17f3446547899b0355cca2b177779889b1245ce4cec7dd0bfcc26de"
  - "repo:crates/hive-cli/src/usage_monitor/threshold.rs#sha256:9f00da30cddf8bbc0083c2e90786e3061fe50947cf41e183c4c8ee9bd91af5c1"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:a5e5881578cd99f1583a00c5a14bca9bc1324a48"
status: active
---

# Usage Guard Stop Layers

The 0.7/0.8 Skills and 0.9 guard block new automatic dispatch. The source-only 15s watcher wrote halt.json for the next gate, never interrupted inference, and was removed in August 2026. Opt-in 0.13.0 reuses sensing/reset data with native Codex user-stop for reset or 60s gaps. 0.13.1 shares the floor policy and owner-bound resume receipts. Windows tests verify reasoning, AI-child and tool-call interruption, notice and same-floor resume. Ordinary external commands may continue; AI children must stop. Earlier shell-exit failures remain failed. Historical inference interruption, other hosts and simultaneous billing termination remain unproven.
