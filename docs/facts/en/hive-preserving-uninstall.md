---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: en
counterpart: ../ko/hive-preserving-uninstall.md
title: "Hive Preserving Uninstall"
summary: "A preserving uninstall removes Hive-owned transient and retired projection state while retaining knowledge, preferences, and foreign bytes."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:665b5cbf2f5f7d0fdb59bbe7515b5264e8cba4a24d2ad980d5401e6965d66d16"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0ec39fe16f77e7403489393b0ca299c93f8d7dc46830f0e7582a283423b6a03f"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:889f7b0e5f374b1c78117486dcd24bd02df5d96b00c340402f2c672eb54b3b61"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/state/CURRENT.md#sha256:01ed5e0aadbcf045f3957b0e1357cdb0d4e922e67a7de57332860c3c3acf0040"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:90a8ecca713a1b1963b5f1863f76d32d5c5b9532ca72922c2705ee9b63520307"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:089b0717e24c368a1725774aaca0c85ab596df10"
status: active
---

# Hive Preserving Uninstall

- Remove Hive activation, projections, package state, indexes, backups, transactions, and runtime.
- Preserve knowledge, saved preferences, and foreign bytes.
- The `test.19` Mac audit found 44 retired empty `agents/` leaves and an empty transaction directory.
- Install, update, and uninstall now prune authenticated Hive-owned empty ancestors leaf-to-root.
- Keep `.hive/dev-install` as separate developer rollback state.
