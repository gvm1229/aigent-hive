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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2fade132e2fe4e60b3f7b70e089d5deaa145a25e2976bdfdb62a58a82fa86192"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3c6e0b3a0f62d5598e68024fd7a69d82cb2b0dcfbb29d5602f6f362bc0df2d40"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/plans/active/cli-uninstall-0.14.0.md#sha256:db50bb091eff11cd8ee1ee39667b992ac7a947a3ea7b0e7d3d74c020f41956d0"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:7565804c77597fea7e64b3c330f9f76031effd27"
status: active
---

# Hive Preserving Uninstall

0.14.0 previews and confirms removal, stops the owned monitor first and preserves knowledge, preferences, project registrations, history and foreign bytes. Unknown ownership stays pending. Isolated OS removal and Windows Codex/monitor recovery passed; public14 artifacts remain separate.

Earlier cleanup scope and test.19 Mac findings (44 empty agents leaves and an empty transaction folder), their exact fixture and 0.9.1 Windows zero-leftover evidence remain in [UOS-021–022](../../archive/plans/foundations/user-onboarding-shared-index.md). Authenticated empty ancestors are pruned leaf-first. Keep `.hive/dev-install` as developer rollback state. In 0.12.1, an enabled monitor required owned-task disable before uninstall.
