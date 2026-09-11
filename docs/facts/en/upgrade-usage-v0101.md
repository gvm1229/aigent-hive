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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:881ae77507817b888cf66ff0da2ee52fbc4048fbb9355641a093dcf6f3d69fc1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6d2138ea9d68803f4447e7295cc08fcf5b548c23df8b2d1e5826c0aa46bff668"
  - "repo:crates/hive-projection/src/lib.rs#sha256:c80df9705880a64e92a2af923ac394fccbdbd19385b4edd8fecfbf7f8c0dce67"
  - "repo:crates/hive-render/src/lib.rs#sha256:b32ec4b80e8196069441c3f6658a6167163b33439ea1859b0349f76b56ccc823"
  - "repo:harness/project-bases/registry.yml#sha256:195103d23c1539bcdb6cd3a84807e52306ed772f8a2fcbbf3f97ab3a56ece039"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:fede7a2a753884a414766773c6cf197721937028"
status: active
---

# Harness Upgrade and Usage Policy Repair in 0.10.1

- Project upgrade: authenticate historical full base, migrate state, then render current output.
- Skill selection: allow declared lifecycle merges; reject duplicates, unknown ids, and tamper.
- `0.9.5`: `iterative-execution|ralph-loop` becomes one `verified-workflow`.
- Registry: exact project/user digests and generated published-prerelease overlay chains, including frozen `v0.10.0` release bytes.
- Usage halt: bind the effective policy digest and recheck a changed threshold in the same session.
- Recheck: remove the exact old halt on allow; replace it on limited or unknown; no guard disable.
- Recovery excludes process changes and unchanged-policy halts; both remain blocked.
