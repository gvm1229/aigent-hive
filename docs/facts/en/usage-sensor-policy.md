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
  - "repo:crates/hive-cli/src/main.rs#sha256:c7dfa1349f5239cd9df726f0fbf7d9a73a3da5a90218551cac420c0d7331a9bd"
  - "repo:crates/hive-cli/src/usage.rs#sha256:b8c12b63eb8011272e75592f2b5ac87a3c6aca78181fe223b6c462b14b332402"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:836d4ac233d8e719a9cd1b5a09a1deee7111a7c4e208f95f07e277fa3c899b98"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:9f1a3ef595af92bb0f83f0cbfda6566923a83fee832dcffaaec9fa05b92af0b1"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:bf97a4aaf3933d9c14eb265a8d4b8b2bcc02e859"
status: active
---

# Usage Sensor Policy

Qualified native reads come first. CodexBar needs consent and an allowlisted unavailable/unsupported result, never a limited result. A missing requested account allows one digest-free retry only for one complete authenticated account. Missing, duplicate, malformed, stale or limited data stays closed. Expedited setup uses 20%; normal setup probes natively before offering fallback. The 0.13.1 monitor's backup is a fresh native process, 5s per path, with the same provider. Windows frozen-reader proof: 1,810s, 115/115 valid, eight query connections, one fallback, zero false 60s gaps. New desktop interruption acceptance is pending.
