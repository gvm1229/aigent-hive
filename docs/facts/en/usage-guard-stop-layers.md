---
schema_version: 1
pair_id: usage-guard-stop-layers
topic_slug: usage-guard-stop-layers
language: en
counterpart: ../ko/usage-guard-stop-layers.md
title: "Usage Guard Stop Layers"
summary: "Released 0.13.1 applies configured floors through native session interruption while preserving the app and normal resume."
tags: [guard, history, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:00874e59d2c865710877e9d224617d7b5d736d76c6f597703ec65b8268261992"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:15d7691b39f9d6e160c2857681d2715ba59ece081f832f9da4df7942ffab6cdf"
  - "repo:crates/hive-cli/src/usage_monitor/threshold.rs#sha256:9c3f5e81cd6ddf7c4177448f63c31f49054bee8b4335a403985d9bbebf85b275"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:docs/research/stable-release-0.13.1.md#sha256:8bb27b7af203a2a4c6a54dc39aa7fd978075e1fb373e6b6d9617e8550acaef4e"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:6096c88095e0ba39fc78dfe959356c2f8d988c2115e3b5519bd55ccf3041464d"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:332849449782b95d9d5819316151e0e8991477fa2cc56ba5df568dcc35f4c237"
  - "repo:tests/results/release-0.13.1-stable.md#sha256:a8c2566122cb825217755f8c9901570e63517e114fbe7a484a7ce6af229e1420"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:ae9ad865350a8e411d89a3bf77d7e6a8433d78302247b7bba51fdde444e918a9"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# Usage Guard Stop Layers

The 0.7/0.8 Skills and 0.9 guard block new automatic dispatch. The source-only 15s watcher wrote halt.json for the next gate, never interrupted inference, and was removed in August 2026. Opt-in 0.13.0 reuses sensing/reset data with native Codex user-stop for reset or 60s gaps. 0.13.1 shares the floor policy and owner-bound resume receipts. The 0.13.1 Windows tests verify reasoning, AI-child and tool-call interruption, notice and same-floor resume. Ordinary external commands may continue; AI children must stop. Earlier shell-exit failures remain failed. Historical inference interruption, other hosts and simultaneous billing termination remain unproven.

The 0.13.2 source checks the authenticated desktop protocol instead of an exact package-version list. Current Windows Codex 26.1016.8120.0 acceptance used controlled observations with real session stops: 5% in 664ms, reset in 1,078ms and zero in 1,161ms. Normal floor/reset resumes passed; a new zero-quota turn stopped in 758ms. Connection, observation and interruption are separate claims. The actual account reset was observed by installed 0.13.1, whose desktop control failed; it is not proof of a 0.13.2 real-reset stop.
