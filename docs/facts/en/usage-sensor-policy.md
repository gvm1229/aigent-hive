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
  - "repo:crates/hive-cli/src/main.rs#sha256:4be37700a3cae8091747c56cb7154d5d9e3be9caa1850df4bf1dd12e725711e7"
  - "repo:crates/hive-cli/src/usage.rs#sha256:ce45a82f02376c71b9bd25a7daf377c11fa1c9b55a4f08098098f97cd7ae920f"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:9b858bef8831bed659b0f977ecd9ba84111800176d9dfc048b1d46c4c6ab8115"
  - "repo:crates/hive-cli/src/usage_control/manual.rs#sha256:86b216d3850bd88b428d1d9ea6407c11c6287f5c3e401926882e8d9e9d48d54d"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9cf945c0072f953b590757cffec4fbda47b68ac0145cb4b573b220aa6ddc4d45"
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:b33ed5b0aa3819b3607100012f444faa6af82f7f1363c2ca492cb7eaf81bb1df"
  - "repo:tests/results/usage-current-app-0.13.1.json#sha256:107a03e8a27f3c3a64de9c34a9b74de7d83f8ced835b318a547d04914c9476c9"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Usage Sensor Policy

Qualified native reads come first. CodexBar needs consent and allowlisted unavailable/unsupported results, never limited usage. A missing requested account permits one digest-free retry only for one complete authenticated account. Missing, duplicate, malformed, stale or limited data blocks enabled protection. Before 0.14.0, quick setup used 20% and setup probed before offering fallback. New v2 protection defaults off; manual usage show only observes. The 0.13.1 monitor uses a fresh native backup, 5s per path, same provider. Windows frozen-reader proof: 1,810s, 115/115 valid, eight connections, one fallback, zero false 60s gaps; reasoning, AI-child, tool and direct-resume checks passed.
