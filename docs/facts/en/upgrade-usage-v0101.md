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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:a5230754ef154b932d4a3be84494ea54f4a82f99d3597c9e31ad787f309c9e3a"
  - "repo:crates/hive-projection/src/lib.rs#sha256:b8167cf34fc47fdba704bc6b56a237ccb3a92161a3111b5690a2036826b93b9c"
  - "repo:crates/hive-render/src/lib.rs#sha256:23b7954dec7567430840b7f9b568bb519df9519693b905c9f7fda19fca13399b"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b8bc06b557cc8582ecf988de4fce0c9c2aefd508d33adabcdb59668d078d988e"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:53634f9e87050790d47ad86c9d0b3332a30ca579b453508d9bf3d21e1084bbd6"
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
