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
  - "repo:docs/guides/branching-rules.md#sha256:76be6ab2fa1ad493840d33974d9b6b30c8b89daa0619f2cdc7f7f21eceda745b"
  - "repo:scripts/branch-policy.py#sha256:e1dd087bf7d2bf9f4057a89959e89690a69486cd05b42fc1ad87c7ee2134f394"
links: [git-worktree-lifecycle]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Source Branch Naming Policy

Only main, develop, or feature/, fix/, release/, docs/, test/, refactor/, build/, and chore/ work branches are allowed. Explicit branch creation authority remains required. One source policy checks syntax, creation, rename, commits, push destinations and PR direction, and generates remote restrictions. Local Git hooks and server naming rules enforce supported boundaries. Git 2.45 native rename can bypass reference hooks; use the rename helper, with commit and push checks as additional guards.
