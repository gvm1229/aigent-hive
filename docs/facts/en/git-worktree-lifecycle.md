---
schema_version: 1
pair_id: git-worktree-lifecycle
topic_slug: git-worktree-lifecycle
language: en
counterpart: ../ko/git-worktree-lifecycle.md
title: "Temporary Git Worktree Lifecycle"
summary: "Maintainer sessions may create needed temporary worktrees; owned worktrees and merged branches require verified cleanup."
tags: [git, workflow, worktree]
aliases: ["temporary clone cleanup", "worktree cleanup"]
sources:
  - "repo:.agents/directives/06-session-coordination.md#sha256:8c889202b65d024691c567253833d3d57f621800c0802ab83d1bdf0df1c0cd20"
  - "repo:.agents/directives/references/git-worktrees.md#sha256:74afce43b00e5e72b0342020cc4022446aee632b0d10f615a5509e31ed41ea7f"
links: [source-development]
reviewed_revision: "git:e032bd9f8a1e404cf473dfd78982c9f7780abb13"
status: active
---

# Temporary Git Worktree Lifecycle

Maintainer sessions may create as many temporary worktrees as parallel or isolated work needs
(maintainer, 2026-11-12). Record ownership and removal boundaries. After work merges or ends,
verify reachability and required pushes, remove owned temporary worktrees, delete merged branches
locally and on origin, then prune. Leave no owned temporary worktree at closure. Preserve dirty
or unmerged work safely; force removal and primary-worktree removal remain prohibited.
