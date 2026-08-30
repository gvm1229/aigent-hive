---
schema_version: 1
pair_id: usage-sensor-policy
topic_slug: usage-sensor-policy
language: en
counterpart: ../ko/usage-sensor-policy.md
title: "Usage Sensor Policy"
summary: "Qualified host-native usage sensors take priority over the optional CodexBar fallback."
tags: [sensor, usage]
aliases: ["Native-first usage"]
sources:
  - "repo:crates/hive-cli/src/main.rs#sha256:024500782daa35d5ab3a6df26a443bf0e4c0653a2a2c19caaa2f1b2a7836cdb6"
  - "repo:crates/hive-cli/src/usage.rs#sha256:c60a6eecaa243ef0528c292303baca85f0bf4c4c4f654612bf97d15fa52ffe69"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6c5febe7ae1ac1a892f7ac412c40d1b8d9ae339fe73fa8153faf9bb22051e1c0"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ab2aaa4dd8d3ec7e90c366a65cf131b6eb2401f1b0b2c95c87d4a6448c7b3bd9"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:a6aea1ed5b977bc818bace5c9d712d2da01328f59753e9b93136c17b1a8f24d3"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:f91816a46d44d57929cb0b580ca32ff4caa95053"
status: active
---

# Usage Sensor Policy

Each host uses a qualified native machine surface first. CodexBar is an optional,
explicitly consented fallback for allowlisted unavailable or unsupported native
results, never a bypass for a native limited decision.

When a supplied Codex account digest is absent, Hive retries the native sensor once
without that digest only when the sensor returns one complete authenticated account.
Missing, duplicate, malformed, stale, or limited results still fail closed and do not
invoke CodexBar.

Expedited setup enables the guard at `20%` remaining. Normal setup names or asks
about CodexBar only after a native-only probe returns an allowlisted failure.
