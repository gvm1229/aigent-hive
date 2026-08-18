---
schema_version: 1
pair_id: automatic-user-projection-refresh
topic_slug: automatic-user-projection-refresh
language: en
counterpart: ../ko/automatic-user-projection-refresh.md
title: "Automatic User Projection Refresh"
summary: "The 0.9.5 direct update validates the mode-specific user-install action and records the public update version transition."
tags: [installation, migration, projection, update]
aliases: ["Post-update projection refresh"]
sources:
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:a757180da3c34992858923db154f6d1f7b8de2d5c353b6bf81a48e32331c19eb"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:edcd380f617cd0879b42ec9e35737a8782c9e162"
status: active
---

# Automatic User Projection Refresh

The direct update refresher requires `InstallHiveUser` for `--apply` and `ValidateHiveUser` for
`--validate`. The public acceptance runner records initial and final binary versions and waits for
the Windows handoff to activate the update.
