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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:4de73afc7e39e6cfb92df9241c16f48f71c2870eedf93b55c6e81317cf30c8cb"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:7459a9e2294fc43dd05bbad91a027287808f619bf5775fde68528a9ea4c6e69a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:3064034c88ce11c947486618e149d977963e8bc15f4a7c3a0acbe1b428f2fcf8"
  - "repo:crates/hive-projection/src/lib.rs#sha256:9561647510a75cdc259d93a93f032e09d620f41adab550291f1c93ef3b564ed4"
  - "repo:crates/hive-render/src/lib.rs#sha256:3a7521d803fb1db33e44d0b24c33453e17c0cc0f9f674aba814b7649cac4c323"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b8bc06b557cc8582ecf988de4fce0c9c2aefd508d33adabcdb59668d078d988e"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:ecd6135583c40d052e63ac2c948832abfbb95dbc8432c791f5035006f2b1b5cf"
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
