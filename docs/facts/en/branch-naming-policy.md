---
schema_version: 1
pair_id: branch-naming-policy
topic_slug: branch-naming-policy
language: en
counterpart: ../ko/branch-naming-policy.md
title: "Source Branch Naming Policy"
summary: "Source branches other than main and develop require a work-class prefix; agent-name prefixes are prohibited."
tags: [branch, git, policy]
aliases: []
sources:
  - "repo:docs/research/branch-naming-audit-2026-09-18.md#sha256:c84ae3cfef5fea40e68e6491930c2b53952eb9211b7f4d0f1b7f21deb785abd8"
links: [git-worktree-lifecycle]
reviewed_revision: "git:f580d1e7cbd2727d02cead3c42af66cd244802c5"
status: active
---

# Source Branch Naming Policy

The maintainer requires source branches other than main and develop to use feature/, fix/, release/, docs/, test/, refactor/, build/, or chore/. Agent-name prefixes such as codex/ are prohibited. A new task branch still requires explicit authorization. Git ref syntax validation does not enforce this policy. The audit proposes aligning the old staging exception and adding enforcement; it does not claim those fixes are implemented.
