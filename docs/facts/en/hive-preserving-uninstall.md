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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2ac47f0ba3f6a05f76c1e524ad9945d695e150c5665ed77dfb496e86ebab82d9"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3164f766afc6f15b3203a96240d7d75f47d95dad9a4d938c73a8866fc4f6f66e"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:aafc3f1fb28a8e43939309d0dfb21586305759e2326d61fce444189f7acc79d1"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/state/CURRENT.md#sha256:c70966fc5c105b51284905879b1ba15b2ab75758fb660712f8b43d9568989006"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:6b2a26d1285073e6796f683abfc190bd6d74a05d57b83900412da37aa5d53849"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:089b0717e24c368a1725774aaca0c85ab596df10"
status: active
---

# Hive Preserving Uninstall

- Remove Hive activation, projections, package state, indexes, backups, transactions, and runtime.
- Preserve knowledge, saved preferences, and foreign bytes.
- The `test.19` Mac audit found 44 retired empty `agents/` leaves and an empty transaction directory.
- Fix with leaf-to-root pruning; keep `.hive/dev-install` as separate developer rollback state.
