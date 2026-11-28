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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:b9439680a397b8459548cffabf6ebd0ff74f5e5ca4977ec8adda5e9676f92a0b"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:71c3c2800d6d88c3366c125a948923d92066cb12f35db02c0594905976d7ac28"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:cd8fc9b3187b4d3395847b367ef455ba6be3b27a7140c720552f76f72eff7f3e"
  - "repo:crates/hive-projection/src/lib.rs#sha256:ce5014879e60d857bb7626250a9bae40f1f9f337c6fb556a675698b9d1cfa1b7"
  - "repo:crates/hive-render/src/lib.rs#sha256:db8acd2d00532f6ed23b0adce6e9c8f3a3648c74874cc860eeb3633b20a0e92d"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b8bc06b557cc8582ecf988de4fce0c9c2aefd508d33adabcdb59668d078d988e"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:6e9425b04b25538acca9e51671d3218789a5d42a44c89a79d9e97c69f06246d3"
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
