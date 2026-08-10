---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: en
counterpart: ../ko/hive-preserving-uninstall.md
title: "Hive Preserving Uninstall"
summary: "hive uninstall removes only Hive-managed user-scope setup state and always retains the knowledge base and saved user preferences."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:f601258ae8aefb9e1456dab1f983272b2074d02b0d862bfe26300afb13f1446b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cb42f6c3bd643bc236f3af89f4388ffdbc08db66af88123a38267b904d7b9d01"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:c28e40cf7e976d0b4455a73c2ddb6d598af448cd549569097784e9a26b2d678e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:4f3676378fafac75f9c6376210c760a2e0200e843ead0825d1b34d7446864e34"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:4a30456"
status: active
---

# Hive Preserving Uninstall

`hive uninstall` removes the exact Hive host activation, Hive projections, Hive package state,
derived index, backups, transactions, and runtime state. It preserves `.hive/knowledge/`, saved
user preferences, foreign host entries, and non-Hive user files. The command has no `--full` or
`-f` mode. Removing knowledge or preferences is a manual, user-owned action.

A later `hive install --scope user --apply` detects the preserved preferences and completes the
user-scope setup without asking the setup questions again.
