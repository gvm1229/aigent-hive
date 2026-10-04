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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:42f69c326667ad73522caeadec761a523074a78efcb5caf05a74185acb0fe3ce"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:fc288bed8f925b89bfd0ed67b808cfdd0722a70b"
status: active
---

# Automatic User Projection Refresh

The direct update refresher requires `InstallHiveUser` for `--apply` and `ValidateHiveUser` for
`--validate`. Public `test.14 → test.15` on M2 macOS recorded the version transition and every
user setup, installation, validation, update-check, update, and final validation result.
