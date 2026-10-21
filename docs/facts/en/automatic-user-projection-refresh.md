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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:d62d3065eb447fe5653b11fbe396490c15471839cb4553c6a3fca1abb368baec"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:ee4aa374f3452d250a376e66c1d1d49ce28c076b95aab8a8bdb9b155e05bca7e"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:docs/research/usage-upgrade-0.13.1.md#sha256:42922efa2e4014307f6936d18fff37cdc359c7cdd5017c3a04f752650f5ea2de"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:9fc4580a60a40af3161efb32b68726d66b41f294"
status: active
---

# Automatic User Projection Refresh

Authenticated existing hosts use `update --apply` and require `UpdateHiveUser`; empty installations bootstrap through `install --apply` and `InstallHiveUser`. Both validate with `ValidateHiveUser`. Public `test.14 → test.15` on M2 macOS recorded the version transition, setup, installation, validation, update check, update and final validation. The 0.13.1 updater also replaces an opted-in monitor through the new user-update executable; it preserves choice/history and reports an incomplete transition as failure.

Public 0.13.0 selected-Skill profiles now authenticate complete known families without requiring unselected Skills; settings, knowledge and history remain intact.
