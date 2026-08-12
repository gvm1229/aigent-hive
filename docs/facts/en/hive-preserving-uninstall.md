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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:4aecfd684f8c07326a639e92061de5f2ea52050cddc352a3b2f4b6b4adb1d3c2"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:756a903f18f96a554c05fdc72b76e0bc515446e3046bea1de6632087e99370aa"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/state/CURRENT.md#sha256:2e46cb683c1f5da90d7c2d7d57ceb08df4492e4052bd51ff14b97e133ab237cc"
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
- The exact 44-leaf fixture and the `0.9.1` Windows preserving reinstall both converged to zero
  retired empty `agents/` leaves and zero empty transaction directories.
- Keep `.hive/dev-install` as separate developer rollback state.
