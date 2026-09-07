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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:3b45af9ce1038d97165445c5a78ad3f354921db983146e83f0313ca6627755e4"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:633265271a82ee37ef07acd9e5a3d406ea80a3f17c7c9809d3d8d7619dd91260"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:d1105d7d3ebc2c5b8dbf100c4383ecd0933a8425c07501785b84ccdd4c6d2719"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
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
