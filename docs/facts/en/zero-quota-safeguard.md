---
schema_version: 1
pair_id: zero-quota-safeguard
topic_slug: zero-quota-safeguard
language: en
counterpart: ../ko/zero-quota-safeguard.md
title: "Legacy Zero Quota Safeguard"
summary: "Legacy exhaustion protection is preserved by migration; new 0.14.0 v2 policies default off."
tags: [exhaustion, session, usage, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:9b858bef8831bed659b0f977ecd9ba84111800176d9dfc048b1d46c4c6ab8115"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:e59ef8b5bfe8b2024bb68a6940e23bf9ea45a4af611d7f5d2078f11c1cdfe228"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/guides/installed-usage-guard.md#sha256:0ead68f33595fcbd18d4d9c9ec38926645ca3cf22ffd895a02628c5e3c97a384"
  - "repo:docs/plans/active/usage-defaults-0.14.0.md#sha256:ce00803f6a0d36c3c7effb56ec5cddea490404d31c0630e869dfb3e1540949cb"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:44f1fb5716b0af3ab556a6d03c38c10d3422a1dc0b2489b79be106f43a62e862"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:4c71dbbc592620844bff81765151014d4a657924d03aea602c08bc517e1c8727"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:76abaa5e2595b9aeee3797fd612694112e9c8efbbcb43ccfaed49653907aeafc"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Legacy Zero Quota Safeguard

0.13.2 accepts 0-99% floors. Zero remains enabled: positive quota permits work; any exhausted window/pool blocks regardless of credits or ordinary disable. Missing, stale or invalid usage blocks. Opt-out needs separate exact host/session/process consent. Fresh positive usage permits rechecking; reset acknowledgement cannot clear exhaustion. Zero receipts are turn-bound; new turns stop again. Observed exhaustion forbids checkpoint, knowledge, lint, Git, cleanup and handoff tools. 0.13.2 Windows controlled stops passed; billing prevention and natural-reset interruption remain unverified. Prior evidence is archived. New 0.14.0 v2 protection defaults off; legacy mappings preserve existing choices.
