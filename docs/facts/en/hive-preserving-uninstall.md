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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9a2ca4c65c5b7880464c7804d224cad91de28493eaf189aa63b1ed439806dde1"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:eb62d9db75f946f948b4dada89fa141654e80135"
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

In 0.12.1, an enabled usage monitor blocks uninstall until its owned scheduled task is disabled.
