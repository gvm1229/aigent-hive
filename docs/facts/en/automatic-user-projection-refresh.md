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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:9488b87b8123e4cfbf702193a6c1ebf066fe143af522c51bea9ba5bdc5b3a6f5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:193eb021ec2fc367176127ae05ab9dc2baea6097819ddd63de43d9a79ee7cc9a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:b7a409e044561428f1bda83e8daef5a647a63c23a8f1d1426baffb336b63323a"
  - "repo:docs/research/usage-upgrade-0.13.1.md#sha256:34eef7d725766ab1de07923d443623df1dd10b4e681950db813488249db2ace4"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Automatic User Projection Refresh

Existing authenticated hosts use `update --apply` and require `UpdateHiveUser`; empty installations bootstrap through `install --apply` and `InstallHiveUser`. Both validate with `ValidateHiveUser`. Public `test.14 → test.15` on M2 macOS recorded the version transition, setup, installation, validation, update check, update and final validation. The 0.13.1 updater also replaces an opted-in monitor through the new user-update executable; it preserves choice/history and reports an incomplete transition as failure.

Public 0.13.0 selected-Skill profiles authenticate complete known families; unselected Skills are optional. Settings, knowledge and history remain intact. Windows monitor handoff waits for the old reader before state conversion.
