---
schema_version: 1
pair_id: zero-quota-safeguard
topic_slug: zero-quota-safeguard
language: en
counterpart: ../ko/zero-quota-safeguard.md
title: "Default-On Zero Quota Safeguard"
summary: "The 0.13.2 source accepts a zero floor and qualifies Windows interruption while retaining independent exhaustion protection."
tags: [exhaustion, session, usage, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6a8b7909e224bde9bcf45ea91e8267219b6a7b1a4546f3b601705db94d144a88"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:3c1f90928debba27a9bb5080e04d4db7b59b15aa32a5dd88b121eb10712624bf"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/guides/installed-usage-guard.md#sha256:735fffd7960d1dca621b1de7bb2fa7f77ef232cda6c75c4165afda76c9f8da50"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:44f1fb5716b0af3ab556a6d03c38c10d3422a1dc0b2489b79be106f43a62e862"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:f901c85a4769dba3021471341fe9a14c0daba392661212bd233d852bfed80304"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:76abaa5e2595b9aeee3797fd612694112e9c8efbbcb43ccfaed49653907aeafc"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# Default-On Zero Quota Safeguard

The 0.13.2 source accepts 0-99% floors. Zero is enabled protection: positive quota may run, but any exhausted window or pool blocks regardless of credits or ordinary disable. Missing, stale or invalid usage blocks work. Opt-out requires separate consent for the exact host/session/process. Fresh positive usage permits rechecking; reset acknowledgement never clears exhaustion. Live zero receipts are turn-bound, so new turns stop again. Observed exhaustion forbids checkpoint, knowledge, lint, Git, cleanup and handoff tools. Current Windows controlled-observation stops are qualified; billing prevention and natural-reset interruption remain unverified. Prior acceptance is archived.
