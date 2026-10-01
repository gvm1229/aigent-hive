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
  - "repo:crates/hive-cli/src/main.rs#sha256:307073fd99899e1aab9aa14bfc92d0fadc1051816763525dd02ab505619424b6"
  - "repo:crates/hive-cli/src/usage.rs#sha256:d53779d091d848cea32a1f79a4a96ad11112f361b032758aed008fb9dbf7d9fa"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6f0d42adfc0aa7cf6e199898d950a7219e0bc3b3c78ef1bbc5b822d742e142dd"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:fc288bed8f925b89bfd0ed67b808cfdd0722a70b"
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
