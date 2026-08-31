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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:8d58b21e0a57a82908a5f6f59e489ec6e17d8e73191b17f9794f3dba16e9aef1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:1518c1b9ac4f68d114a59603a490491221b0459e36137fb380d2c247f9e1ab1a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:acd4022de5697806003207634ac0b7cb874baeb802af491f28d39ec048daf830"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# Automatic User Projection Refresh

The direct update refresher requires `InstallHiveUser` for `--apply` and `ValidateHiveUser` for
`--validate`. Public `test.14 → test.15` on M2 macOS recorded the version transition and every
user setup, installation, validation, update-check, update, and final validation result.
