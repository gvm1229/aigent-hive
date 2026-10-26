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
  - "repo:docs/plans/active/cli-uninstall-0.14.0.md#sha256:bce48a3b7a2443fffb3177e708e64f061f47a4942ac37a66445099069ab91767"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:14b97538abd3ac185627d5dc0a573b540122f780e5549edf80373c6c98866e69"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:8c396a991cefeff19dc4a745d91caf7f2ef7c723"
status: active
---

# Hive Preserving Uninstall

0.14.0 previews and confirms removal, stops the owned monitor first, and preserves knowledge, preferences, project registrations, history and foreign bytes. Unknown program ownership stays pending; actual OS removal acceptance remains open.

Earlier cleanup scope and test.19 Mac findings (44 empty agents leaves and an empty transaction folder), their exact fixture and 0.9.1 Windows zero-leftover evidence remain in [UOS-021–022](../../archive/plans/foundations/user-onboarding-shared-index.md). Authenticated empty ancestors are pruned leaf-first. Keep `.hive/dev-install` as developer rollback state. In 0.12.1, an enabled monitor required owned-task disable before uninstall.
