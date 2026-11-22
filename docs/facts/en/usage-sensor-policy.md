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
  - "repo:crates/hive-cli/src/main.rs#sha256:9ba3432d1374deb45c60bdd6a6b7b0ad52cf2f98d3274c55c5ee24c8d86c09e1"
  - "repo:crates/hive-cli/src/usage.rs#sha256:ce45a82f02376c71b9bd25a7daf377c11fa1c9b55a4f08098098f97cd7ae920f"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:50bbe234dff657b08dca42bc8cb17330a75d6aee436b5debd318fb039fd787c4"
  - "repo:crates/hive-cli/src/usage_control/manual.rs#sha256:56dfd0b134074e7797d3ab27b203800fce2809758460a40f0500e1ad7b39bd80"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:33b0a59a13b659d8b98b583e682316b8df27964f7efc798af34967c8411cfe9e"
  - "repo:crates/hive-core/src/preferences.rs#sha256:6ec5ee3711750ed82cd015e412ade49185152bec1831dbcbf077e52f122cce3b"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:tests/results/usage-current-app-0.13.1.json#sha256:107a03e8a27f3c3a64de9c34a9b74de7d83f8ced835b318a547d04914c9476c9"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Usage Sensor Policy

Qualified native reads come first. CodexBar needs consent and allowlisted unavailable/unsupported results, never limited usage. A missing requested account permits one digest-free retry only for one complete authenticated account. Missing, duplicate, malformed, stale or limited data blocks enabled protection. Before 0.14.0, quick setup used 20% and setup probed before offering fallback. New v2 protection defaults off; manual usage show only observes. The 0.13.1 monitor uses a fresh native backup, 5s per path, same provider. Windows frozen-reader proof: 1,810s, 115/115 valid, eight connections, one fallback, zero false 60s gaps; reasoning, AI-child, tool and direct-resume checks passed.
