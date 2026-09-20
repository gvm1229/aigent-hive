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
  - "repo:docs/guides/branching-rules.md#sha256:3820e9a509c4eaee972e2900550d03d2f1a8f7140c8dbc3f6adf1c55596a00ae"
  - "repo:scripts/branch-policy.py#sha256:3be4d5412e7c4060636f93f612b1df9b118ff9fa54d980e805a6362fdbb3f1f7"
links: [git-worktree-lifecycle]
reviewed_revision: "git:a642bd91e1cabdf80673b836f2e801a2c6cf9b61"
status: active
---

# Source Branch Naming Policy

Only main, develop, or feature/, fix/, release/, docs/, test/, refactor/, build/, and chore/ work branches are allowed. Explicit branch creation authority remains required. One source policy checks syntax, creation, rename, commits, push destinations and PR direction, and generates remote restrictions. Local Git hooks and server naming rules enforce supported boundaries. Git 2.45 native rename can bypass reference hooks; use the rename helper, with commit and push checks as additional guards.
