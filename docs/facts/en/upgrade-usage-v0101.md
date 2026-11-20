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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:81c8223df0cba01511f0da69e8bc2b79bca26a22fa0439af874f6b4ed320606e"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:bebefffbf834ee4eefaeb371f1041e9a74a9eaba1aa51368461d5a425fedeb42"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:961f63ac2939655e614a95c1c0a950d86cc7cbb753402e78bf8e53674b0f9dca"
  - "repo:crates/hive-projection/src/lib.rs#sha256:2b119cea5c4b445e65dd63fb32a7a77ecb983b9579522dfdd0ec4dfa9e5ae7a3"
  - "repo:crates/hive-render/src/lib.rs#sha256:796ea6a394afd04562c97e4e00275e83afd30fe9d208d1f6fb1dfbb08b38fa7c"
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
