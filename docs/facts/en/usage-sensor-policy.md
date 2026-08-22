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
  - "repo:crates/hive-cli/src/main.rs#sha256:edc9e588f4303932f8323ec26f58f00481da7585ae7334c4b8d5048959aa7e20"
  - "repo:crates/hive-cli/src/usage.rs#sha256:c60a6eecaa243ef0528c292303baca85f0bf4c4c4f654612bf97d15fa52ffe69"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6c5febe7ae1ac1a892f7ac412c40d1b8d9ae339fe73fa8153faf9bb22051e1c0"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:786da31401085e9445495aa37defe7cedf781bc8457211a6addd23016c0bf922"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:1fcbb2b9b2db6d57bd40682f80db2a0a916ebbffb3434431038b609b6b743c11"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
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
