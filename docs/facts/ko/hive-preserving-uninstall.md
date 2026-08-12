---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: ko
counterpart: ../en/hive-preserving-uninstall.md
title: "Hive 보존형 제거"
summary: "보존형 제거: Hive-owned transient·retired projection 완전 정리, knowledge·preference·foreign byte 보존."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2ac47f0ba3f6a05f76c1e524ad9945d695e150c5665ed77dfb496e86ebab82d9"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3164f766afc6f15b3203a96240d7d75f47d95dad9a4d938c73a8866fc4f6f66e"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:aafc3f1fb28a8e43939309d0dfb21586305759e2326d61fce444189f7acc79d1"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/state/CURRENT.md#sha256:2a541e1d5843a67fa8835adf7efaaef6dc7bd13409e603903789378d3eb84e07"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:6b2a26d1285073e6796f683abfc190bd6d74a05d57b83900412da37aa5d53849"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:089b0717e24c368a1725774aaca0c85ab596df10"
status: active
---

# Hive 보존형 제거

- 제거: Hive activation·projection·package·index·backup·transaction·runtime
- 보존: knowledge·saved preference·foreign byte
- `test.19` Mac audit: retired empty `agents/` leaf 44개와 empty transaction directory 잔존
- 보정: leaf-to-root prune, owned empty shell `0건`
- `.hive/dev-install`: 별도 developer rollback state
