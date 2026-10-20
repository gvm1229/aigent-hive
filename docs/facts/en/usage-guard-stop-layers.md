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
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:b8e7b8f90c305c715a2cd7a0e4e1fc89133ad57f8f1fbfe8aeef2e8efe62d20e"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:798f0e43b17f3446547899b0355cca2b177779889b1245ce4cec7dd0bfcc26de"
  - "repo:crates/hive-cli/src/usage_monitor/threshold.rs#sha256:9f00da30cddf8bbc0083c2e90786e3061fe50947cf41e183c4c8ee9bd91af5c1"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:20d7ea0fbdbfa3cb34bb0bd0548453660d556d628eda16a609e15a2e8ff562a9"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
status: active
---

# Usage Guard Stop Layers

The 0.7/0.8 Skills and 0.9 guard block new automatic dispatch. The source-only 15s watcher wrote halt.json for the agent's next gate, not native interruption; it was removed in August 2026. The opt-in 0.13.0 monitor reuses sensors/reset data and adds independent supervision plus native Codex user-stop on reset or a 60s observation gap. Development 0.13.1 connects the shared floor policy to that actuator, with owner-scoped resume receipts. Earlier successful gates do not prove interruption inside one inference. New long-reasoning/tool acceptance is pending; Windows proof does not establish other-host support.
