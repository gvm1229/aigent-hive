---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: en
counterpart: ../ko/hive-preserving-uninstall.md
title: "Hive Preserving Uninstall"
summary: "Confirmed removal stops the owned monitor and preserves settings, knowledge, registrations and history."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:cd8fc9b3187b4d3395847b367ef455ba6be3b27a7140c720552f76f72eff7f3e"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:04bab46c9c8e3aa4b7491d6ff2e3051d6fa5b4b4a16fc79e46808a6e3e619308"
  - "repo:docs/archive/plans/cli-uninstall-0.14.0.md#sha256:db50bb091eff11cd8ee1ee39667b992ac7a947a3ea7b0e7d3d74c020f41956d0"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Hive Preserving Uninstall

0.14.0 previews and confirms removal, stops the owned monitor first and preserves knowledge, preferences, project registrations, history and foreign bytes. Unknown ownership stays pending. Isolated OS removal and Windows Codex/monitor recovery passed; public14 artifacts remain separate.

Earlier cleanup scope and test.19 Mac findings (44 empty agents leaves and an empty transaction folder), their exact fixture and 0.9.1 Windows zero-leftover evidence remain in [UOS-021–022](../../archive/plans/foundations/user-onboarding-shared-index.md). Authenticated empty ancestors are pruned leaf-first. Keep `.hive/dev-install` as developer rollback state. In 0.12.1, an enabled monitor required owned-task disable before uninstall.
