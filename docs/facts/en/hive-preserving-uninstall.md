---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: en
counterpart: ../ko/hive-preserving-uninstall.md
title: "Hive Preserving Uninstall"
summary: "A structurally valid user-scope ownership manifest that mismatches an authenticated release triggers an automatic preserving reinstall that retains the knowledge base and saved user preferences."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d24cc8e55c8706144ac684cb7ccce3bfa9119c4bd0e20a3e6e36222d9d731eea"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0bfd9117a0d835da5f19bc02b82959a5630a4955a81ee3efda0a6ba5246dfaad"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:6b2a26d1285073e6796f683abfc190bd6d74a05d57b83900412da37aa5d53849"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:089b0717e24c368a1725774aaca0c85ab596df10"
status: active
---

# Hive Preserving Uninstall

`hive uninstall` removes Hive activation, projections, package state, index, backups,
transactions, and runtime. It preserves `.hive/knowledge/`, saved preferences, foreign entries,
and non-Hive files. No `--full` or `-f`; knowledge and preference removal stay user-owned.

`hive install --scope user --apply` restores projections from saved preferences without setup
questions. A structurally valid manifest that mismatches an authenticated release triggers an
already-authorized install, update, or setup preserving reinstall without another approval.
Malformed or path-unsafe manifests, foreign overwrites, and material choices require a user decision.
