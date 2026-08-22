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
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:4051c30172386a0ce34b86451609e5504fea3dfda7436c5a16bd1680838c1585"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:c95242b313d9abeaee81bd3c89d69af927725c44"
status: active
---

# Automatic User Projection Refresh

The direct update refresher requires `InstallHiveUser` for `--apply` and `ValidateHiveUser` for
`--validate`. Public `test.14 → test.15` on M2 macOS recorded the version transition and every
user setup, installation, validation, update-check, update, and final validation result.
