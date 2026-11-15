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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:d3a387b8364633333832c10b7c4e682b09f2343b916957b0e63d555eb3999fb4"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:ccdb3fbd321eef3dd9d91c412832203b68fd71bab30689c8a9095f6fd89520f3"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:307f40fbc24d404398dc5852f746f03f8599379fd22716f54cfc1ae13a3600af"
  - "repo:crates/hive-projection/src/lib.rs#sha256:05ece29f62af952b3a7d30fc5fa38a279d161bdde8e8f608cc1924e37c49a699"
  - "repo:crates/hive-render/src/lib.rs#sha256:395e1b02d6e50e6460f9c05b0765b5329a6a3dc995de0c102bf5f8edebfe7a78"
  - "repo:docs/guides/installed-usage-guard.md#sha256:96e73fe5c33bb18cf1c2d6e21271df1738c02bc77f06a70fadbcb5b67ea8f5e5"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:d686ec7a433db992078e0428e25ecf337cae6d9a1a37f341dfe5ac37274bf284"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
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
