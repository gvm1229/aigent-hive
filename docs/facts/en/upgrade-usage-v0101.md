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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:50bbe234dff657b08dca42bc8cb17330a75d6aee436b5debd318fb039fd787c4"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:55ef30622eec809540d234114905762323bbf940761d41aa0d7e75df67fd9c57"
  - "repo:crates/hive-projection/src/lib.rs#sha256:231667ff59ebdf77e225a02c36e12a911a4aa4ce23eb5290d94f7f560306b662"
  - "repo:crates/hive-render/src/lib.rs#sha256:bd58a2ef9a317149e6c76b0d23632f67174e139a9f339770247ffb458e0e9c39"
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
