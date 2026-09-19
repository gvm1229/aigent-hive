---
schema_version: 1
pair_id: upgrade-usage-v0101
topic_slug: upgrade-usage-v0101
language: en
counterpart: ../ko/upgrade-usage-v0101.md
title: "Harness Upgrade and Usage Policy Repair in 0.10.1"
summary: "0.10.1 authenticates historical project state before migration and rechecks changed usage thresholds without disabling the guard."
tags: [migration, project-upgrade, usage, v0-10-1]
aliases: ["0.10.1 upgrade repair"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:d5e36f5d1cb6080fa7952b1cf4354e7d54f0df12bc0799d758ced53d7f083b84"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:267309f9d7f56095b6cc00f4602aeaf1249e0e8988f7fc5bd897dff04e2be5f5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:crates/hive-projection/src/lib.rs#sha256:72d1a5158093bb93070183b12db1d2ce8388fd274f6b9a1fd21188ef7bac04b1"
  - "repo:crates/hive-render/src/lib.rs#sha256:d7ac376de1ffbfdf6f04f900fa8b46d78749755ffe9c30e1710be42611353ef7"
  - "repo:docs/guides/installed-usage-guard.md#sha256:9cf01b711909bca15472e95b5a25325b7094df4115795b6fdf04cf2bc3f017f5"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:0184c52dee665e90424d64cafa8c0a76e673d4a43019549e138879cc2085cafc"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:a1b96c88b4938c8771ccf635ffef0e8dcafa9ff1"
status: active
---

# Harness Upgrade and Usage Policy Repair in 0.10.1

- Project upgrade: authenticate historical full base, migrate state, then render current output.
- Skill selection: allow declared lifecycle merges; reject duplicates, unknown ids, and tamper.
- `0.9.5`: `iterative-execution|ralph-loop` becomes one `verified-workflow`.
- Registry: exact project/user digests and generated published-prerelease overlay chains, including frozen `v0.10.0` release bytes.
- Usage halt: bind the effective policy digest and recheck a changed threshold in the same session.
- Recheck: remove the exact old halt on allow; replace it on limited or unknown; no guard disable.
- The old same-process recovery limit was superseded by 0.10.3 remeasurement; 0.11.0 reset acknowledgement is separate.
