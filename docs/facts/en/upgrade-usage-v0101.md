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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:30e453d8375c22b0ec9ec56998fc472dace3fff8cea63ad7ad360d5ff4c19898"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:836d4ac233d8e719a9cd1b5a09a1deee7111a7c4e208f95f07e277fa3c899b98"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:ee4aa374f3452d250a376e66c1d1d49ce28c076b95aab8a8bdb9b155e05bca7e"
  - "repo:crates/hive-projection/src/lib.rs#sha256:d9ec515a86f7c86a760de68c0b68edc70e4c11bdcb79c42524b403afd1218767"
  - "repo:crates/hive-render/src/lib.rs#sha256:f68d23e2eb431d0a36193df9c26d59db5fdc8d00afc9dc00bf1588175e486c0c"
  - "repo:docs/guides/installed-usage-guard.md#sha256:735fffd7960d1dca621b1de7bb2fa7f77ef232cda6c75c4165afda76c9f8da50"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:dfd706527e2688b5c64213c1fa69f7f19dce5904ef150b54c856bc0d743050ec"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:9fc4580a60a40af3161efb32b68726d66b41f294"
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
