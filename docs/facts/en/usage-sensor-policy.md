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
  - "repo:crates/hive-cli/src/main.rs#sha256:09ec7e1bf7616d3cd01ac2d38a87f9cfdf6d0fb1435f304f8e36e7c934ae0b90"
  - "repo:crates/hive-cli/src/usage.rs#sha256:d15d172496becbc6623d561a8fab0729364f11bb4b9a1ce4b2c3069ff23406c3"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:50d537ee8e11e355b035701d3b3f17369c0bc304"
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
