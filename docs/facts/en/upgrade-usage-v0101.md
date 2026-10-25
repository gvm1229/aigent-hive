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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:1ab23dbb5a8b60322f6c1068ba7cc09f54bc34151915068a748a1cc5a2c38018"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:9b858bef8831bed659b0f977ecd9ba84111800176d9dfc048b1d46c4c6ab8115"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2fade132e2fe4e60b3f7b70e089d5deaa145a25e2976bdfdb62a58a82fa86192"
  - "repo:crates/hive-projection/src/lib.rs#sha256:2db910e1905f55a1ed8611722fcadb091f38a19de2f1dddd948d3f5647232b7c"
  - "repo:crates/hive-render/src/lib.rs#sha256:ae4532f02e5c3bd00a88534dfb974cbda2a7d6c02d5b43f43d50e156a9137904"
  - "repo:docs/guides/installed-usage-guard.md#sha256:0ead68f33595fcbd18d4d9c9ec38926645ca3cf22ffd895a02628c5e3c97a384"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:cbe651209f49fcac5828a61799af1562ea0d16978eaa545e739207a386ccafb6"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
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
