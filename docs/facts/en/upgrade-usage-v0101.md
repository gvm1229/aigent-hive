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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:a9535c1ae9e207b08dfce0d71fe9293e38168d8c7a38a4187ce8c6752be90ce4"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:66dc337fae3ced831c3775915aef1b83c6406314cee873f20cf81e75d3c826cf"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:5a7d279c9ce96bec6792f191e4be0caa69bf17c9062ad535e11478af7f4408c2"
  - "repo:crates/hive-projection/src/lib.rs#sha256:274d1221abb312197451cd8afc55a45eda881d08980b932d87454659b46c562d"
  - "repo:crates/hive-render/src/lib.rs#sha256:4ce1a5feac500ede6f71c6b1b2ba0764e189ab48b6cae971122e8d5e538eee42"
  - "repo:harness/project-bases/registry.yml#sha256:3e53d7ee8a72c0dfff784927fa8a22ecbfef6f19dd0f77d76d3772c5043d6779"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:ecd92340604353fd0935e8f0c4ccdcd2b34288f7"
status: active
---

# Harness Upgrade and Usage Policy Repair in 0.10.1

- Project upgrade: authenticate historical full base, migrate state, then render current output.
- Skill selection: allow declared lifecycle merges; reject duplicates, unknown ids, and tamper.
- `0.9.5`: `iterative-execution|ralph-loop` becomes one `verified-workflow`.
- Registry: exact project/user digests, including frozen `v0.10.0` release bytes.
- Usage halt: bind the effective policy digest and recheck a changed threshold in the same session.
- Recheck: remove the exact old halt on allow; replace it on limited or unknown; no guard disable.
