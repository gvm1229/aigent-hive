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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:71c3c2800d6d88c3366c125a948923d92066cb12f35db02c0594905976d7ac28"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:4fd5dd55857ac73d960d702e697b1cd05502f68806844c17c1bd199f8d1f4021"
  - "repo:crates/hive-core/src/preferences.rs#sha256:6ec5ee3711750ed82cd015e412ade49185152bec1831dbcbf077e52f122cce3b"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:d292bfa14d8366b658c2c8ef5abb03900b8a869f66e8de3c14e714783fc34626"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/archive/plans/usage-defaults-0.14.0.md#sha256:61da2591b7e2adbd852472c1ccbab817e695ca424f0ae35f80bc196a77ea8d99"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b8bc06b557cc8582ecf988de4fce0c9c2aefd508d33adabcdb59668d078d988e"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:44f1fb5716b0af3ab556a6d03c38c10d3422a1dc0b2489b79be106f43a62e862"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:d263c84680b1b11fc49175ee181955a2b18fd90e819b1929212fd818b176e3fd"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:76abaa5e2595b9aeee3797fd612694112e9c8efbbcb43ccfaed49653907aeafc"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Legacy Zero Quota Safeguard

0.13.2 accepts 0-99% floors. Zero remains enabled: positive quota permits work; any exhausted window/pool blocks regardless of credits or ordinary disable. Missing, stale or invalid usage blocks. Opt-out needs separate exact host/session/process consent. Fresh positive usage permits rechecking; reset acknowledgement cannot clear exhaustion. Zero receipts are turn-bound; new turns stop again. Observed exhaustion forbids checkpoint, knowledge, lint, Git, cleanup and handoff tools. 0.13.2 Windows controlled stops passed; billing prevention and natural-reset interruption remain unverified. Prior evidence is archived. New 0.14.0 v2 protection defaults off; legacy mappings preserve existing choices.
