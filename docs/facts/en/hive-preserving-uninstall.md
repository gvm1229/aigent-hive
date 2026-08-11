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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b31b041d147e54b8c771e8f89b4dab62282474a5c2761aa3eb41402d5cfda7ff"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0bfd9117a0d835da5f19bc02b82959a5630a4955a81ee3efda0a6ba5246dfaad"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:6b2a26d1285073e6796f683abfc190bd6d74a05d57b83900412da37aa5d53849"
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
