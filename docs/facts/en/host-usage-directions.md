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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b830054a7de89161421441148b7683d339ab383d31f3a1441690556a10fd2daf"
  - "repo:crates/hive-cli/src/usage_control/antigravity.rs#sha256:16907c56509e4dbc868d045112af64c55cbb402d227bc5384bb254cf0ae8fe6a"
  - "repo:crates/hive-cli/src/user_setup/setup_engine/host_windows.rs#sha256:26016906f758d48f982e3f378034b569d079b3c3448c738dba0cf4276c408420"
  - "repo:crates/hive-core/src/preferences.rs#sha256:6ec5ee3711750ed82cd015e412ade49185152bec1831dbcbf077e52f122cce3b"
links: [usage-guard-thresholds, usage-sensor-policy]
reviewed_revision: "git:9c19940d54e90822e384a95b9482071620324e98"
status: active
---

# Host Usage Directions

`HOST_USAGE_SENSORS` is the shared source for each host's native percentage direction, supported windows and sensor version range. Claude accepts used percentage; the CLI stores its inverse as remaining percentage. Codex and Antigravity accept remaining percentage. A host threshold update needs an existing `host_windows` entry and changes only selected guarded windows. Used thresholds below 50 require `--confirm`. Antigravity callback versions from 1.1.18 up to but excluding 2.0.0 are accepted. The real 1.2.17 callback remains unverified because the local fixture is missing; its test stays ignored.
