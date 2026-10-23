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
  - "repo:crates/hive-cli/src/main.rs#sha256:53492315e69a8d63cfa180d672e2557eb99b9102a7606998462eb334f192dd8f"
  - "repo:crates/hive-cli/src/usage.rs#sha256:b8c12b63eb8011272e75592f2b5ac87a3c6aca78181fe223b6c462b14b332402"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6a8b7909e224bde9bcf45ea91e8267219b6a7b1a4546f3b601705db94d144a88"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a7caad0d660d83378b297ec3678312fa741b411b8115ed9eca98400d1db703d0"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:tests/results/usage-current-app-0.13.1.json#sha256:107a03e8a27f3c3a64de9c34a9b74de7d83f8ced835b318a547d04914c9476c9"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# Usage Sensor Policy

Qualified native reads come first. CodexBar needs consent and an allowlisted unavailable/unsupported result, never a limited result. A missing requested account allows one digest-free retry only for one complete authenticated account. Missing, duplicate, malformed, stale or limited data stays closed. Expedited setup uses 20%; normal setup probes natively before offering fallback. The 0.13.1 monitor's backup is a fresh native process, 5s per path, with the same provider. Windows frozen-reader proof: 1,810s, 115/115 valid, eight query connections, one fallback, zero false 60s gaps. The 0.13.1 Windows reasoning, AI-child, tool and direct-resume checks pass.
