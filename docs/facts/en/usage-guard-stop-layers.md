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
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:39d24941f041d49e4aaee3c9130d0e430a7a14a5328557701e223dc93299aa65"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:7c60015123b18b54f4b2a1984839d5a3e5fc96b4935b8ded0ba7a6367d41279d"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:11435eda675af021821e84d452ade72efdc18b2e3829aa67ac7af88ecdcc09bd"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:914c06a3a071de37d529ab6ca4626f5cf548e129"
status: active
---

# Usage Guard Stop Layers

The 0.7/0.8 fixed Skills and 0.9 product guard block new automatic dispatch. The old source-only 15s watcher wrote halt.json; agents stopped at the next gate, not through a native turn interrupt. It was removed in August 2026. The opt-in 0.13 monitor reuses usage sensors and reset data, but adds independent supervision and native Codex user-stop requests. Its live triggers are reset and a 60s observation gap; ordinary remaining-quota thresholds are not connected to that actuator. Past successful gate stops do not prove interruption during one inference. Windows evidence does not establish other-host support.
