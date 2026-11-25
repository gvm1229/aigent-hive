---
schema_version: 1
pair_id: host-usage-directions
topic_slug: host-usage-directions
language: en
counterpart: ../ko/host-usage-directions.md
title: "Host Usage Directions"
summary: "One host sensor table sets accepted percentage direction, guarded windows and supported versions."
tags: [guard, sensor, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:50bbe234dff657b08dca42bc8cb17330a75d6aee436b5debd318fb039fd787c4"
  - "repo:crates/hive-cli/src/usage_control/antigravity.rs#sha256:ead5c18719be74afaaebb8fca8fd580e9aff48d018fc4bab4df1d51032aef909"
  - "repo:crates/hive-cli/src/user_setup/setup_engine/host_windows.rs#sha256:41c24ecb384dbc3102eb8836946b37bf9ad33a463ffdb16abd8170eb591e45c2"
  - "repo:crates/hive-core/src/preferences.rs#sha256:6ec5ee3711750ed82cd015e412ade49185152bec1831dbcbf077e52f122cce3b"
links: [usage-guard-thresholds, usage-sensor-policy]
reviewed_revision: "git:9c19940d54e90822e384a95b9482071620324e98"
status: active
---

# Host Usage Directions

`HOST_USAGE_SENSORS` is the shared source for each host's native percentage direction, supported windows and sensor version range. Claude accepts used percentage; the CLI stores its inverse as remaining percentage. Codex and Antigravity accept remaining percentage. A host threshold update needs an existing `host_windows` entry and changes only selected guarded windows. Used thresholds below 50 require `--confirm`. Antigravity callback versions from 1.1.18 up to but excluding 2.0.0 are accepted. The real 1.2.17 callback remains unverified because the local fixture is missing; its test stays ignored.
