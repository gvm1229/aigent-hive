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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:c32316f67b6846b57cfd654b4abd8a518c74244cb6be1a9981e135190e203a07"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:74e1947063fa479fd0e267683e1ca73621e03a1fd86b05ca544d9ff64b23a242"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:877821f19b0a373dda8202589d1c2eee892eca0c3b0026bc4b15556307307600"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:fc288bed8f925b89bfd0ed67b808cfdd0722a70b"
status: active
---

# Automatic User Projection Refresh

The direct update refresher requires `InstallHiveUser` for `--apply` and `ValidateHiveUser` for
`--validate`. Public `test.14 → test.15` on M2 macOS recorded the version transition and every
user setup, installation, validation, update-check, update, and final validation result.
