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
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:5200ab01acbf0c0577e27de976b91c5a697dd83437a25ed94de3ec93c510dcf3"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/state/CURRENT.md#sha256:7b6375b763404a4b17a4aedd9e06b3961fa4be4813ddece585b960e7efb53363"
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
