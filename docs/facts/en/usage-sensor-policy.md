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
  - "repo:crates/hive-cli/src/main.rs#sha256:c450fea38404d374024d407dadd559b4f847cc13a7c9ed54e21aac50d931a787"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:5200ab01acbf0c0577e27de976b91c5a697dd83437a25ed94de3ec93c510dcf3"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:5602025a4eb182cc6e51cc816cab74983f10ee2bdd2f6324649de63fdbddef1f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:90a8ecca713a1b1963b5f1863f76d32d5c5b9532ca72922c2705ee9b63520307"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Usage Sensor Policy

Each host uses a qualified native machine surface first. CodexBar is an optional,
explicitly consented fallback for allowlisted unavailable or unsupported native
results, never a bypass for a native limited decision.

Expedited setup enables the guard at `20%` remaining. Normal setup names or asks
about CodexBar only after a native-only probe returns an allowlisted failure.
