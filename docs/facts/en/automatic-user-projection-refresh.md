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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d16fe3cf14d23b5781bbb960884146b17c3770916a5572215e4922a4423f28d1"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
status: active
---

# Automatic User Projection Refresh

Authenticated existing hosts use `update --apply` and require `UpdateHiveUser`; empty installations bootstrap through `install --apply` and `InstallHiveUser`. Both validate with `ValidateHiveUser`. Public `test.14 → test.15` on M2 macOS recorded the version transition, setup, installation, validation, update check, update and final validation. The 0.13.1 updater also replaces an opted-in monitor through the new user-update executable; it preserves choice/history and reports an incomplete transition as failure.
