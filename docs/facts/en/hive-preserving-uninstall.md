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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:4790c9666065f4bc49ebf0eaee4c50fce384a2fd44a69cd16670b9c9d6d7f39a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:2dbd0f956fea6c6e258a275bc89565c48a7bf211819ea8816512215dc2582213"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:4f61520861d38b63448a45b91dd96443dfba20c79b3d8abade6099460956d3ed"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:4f3676378fafac75f9c6376210c760a2e0200e843ead0825d1b34d7446864e34"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
status: active
---

# Hive Preserving Uninstall

`hive uninstall` removes the exact Hive host activation, Hive projections, Hive package state,
derived index, backups, transactions, and runtime state. It preserves `.hive/knowledge/`, saved
user preferences, foreign host entries, and non-Hive user files. The command has no `--full` or
`-f` mode. Removing knowledge or preferences is a manual, user-owned action.

A later `hive install --scope user --apply` detects the preserved preferences, restores the
Hive-owned user projection, and completes the user-scope setup without setup questions.
