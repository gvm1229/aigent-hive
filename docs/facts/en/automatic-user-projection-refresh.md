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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:cd9973f27fdd4952ee1936257cb4ed0f2f98d6b8033aa1f89fc2b93eefeb34fd"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a7caad0d660d83378b297ec3678312fa741b411b8115ed9eca98400d1db703d0"
  - "repo:docs/research/usage-upgrade-0.13.1.md#sha256:34eef7d725766ab1de07923d443623df1dd10b4e681950db813488249db2ace4"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:0ee29cc7a10c6786f5d325569f85182e654cd4b1"
status: active
---

# Automatic User Projection Refresh

Existing authenticated hosts use `update --apply` and require `UpdateHiveUser`; empty installations bootstrap through `install --apply` and `InstallHiveUser`. Both validate with `ValidateHiveUser`. Public `test.14 → test.15` on M2 macOS recorded the version transition, setup, installation, validation, update check, update and final validation. The 0.13.1 updater also replaces an opted-in monitor through the new user-update executable; it preserves choice/history and reports an incomplete transition as failure.

Public 0.13.0 selected-Skill profiles authenticate complete known families; unselected Skills are optional. Settings, knowledge and history remain intact. Windows monitor handoff waits for the old reader before state conversion.
