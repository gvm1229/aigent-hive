---
schema_version: 1
pair_id: usage-sensor-policy
topic_slug: usage-sensor-policy
language: en
counterpart: ../ko/usage-sensor-policy.md
title: "Usage Sensor Policy"
summary: "Qualified native reads precede consented fallback; disabled v2 protection and manual observation remain distinct."
tags: [sensor, usage]
aliases: ["Native-first usage"]
sources:
  - "repo:crates/hive-cli/src/main.rs#sha256:640beb2c620260bf5be50a347701c36cf141440b97535b51a2e1e894f2aa1ba3"
  - "repo:crates/hive-cli/src/usage.rs#sha256:ce45a82f02376c71b9bd25a7daf377c11fa1c9b55a4f08098098f97cd7ae920f"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:ccdb3fbd321eef3dd9d91c412832203b68fd71bab30689c8a9095f6fd89520f3"
  - "repo:crates/hive-cli/src/usage_control/manual.rs#sha256:56dfd0b134074e7797d3ab27b203800fce2809758460a40f0500e1ad7b39bd80"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:34e8f7c0d827e90ed98a1b8832f8a2ab4b19f736ea67135f6fe71f728bf87287"
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
  - "repo:tests/results/usage-current-app-0.13.1.json#sha256:107a03e8a27f3c3a64de9c34a9b74de7d83f8ced835b318a547d04914c9476c9"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Usage Sensor Policy

Qualified native reads come first. CodexBar needs consent and allowlisted unavailable/unsupported results, never limited usage. A missing requested account permits one digest-free retry only for one complete authenticated account. Missing, duplicate, malformed, stale or limited data blocks enabled protection. Before 0.14.0, quick setup used 20% and setup probed before offering fallback. New v2 protection defaults off; manual usage show only observes. The 0.13.1 monitor uses a fresh native backup, 5s per path, same provider. Windows frozen-reader proof: 1,810s, 115/115 valid, eight connections, one fallback, zero false 60s gaps; reasoning, AI-child, tool and direct-resume checks passed.
