---
schema_version: 1
pair_id: git-worktree-lifecycle
topic_slug: git-worktree-lifecycle
language: en
counterpart: ../ko/git-worktree-lifecycle.md
title: "Temporary Git Worktree Lifecycle"
summary: "Ordinary work uses one primary worktree; authorized temporary worktrees require immediate verified cleanup."
tags: [git, workflow, worktree]
aliases: ["temporary clone cleanup", "worktree cleanup"]
sources:
  - "repo:.agents/directives/03-workflow.md#sha256:629d32bb289108bbc782e295e4ffda6a4a4d5006fbf151212db0cc79457391f0"
  - "repo:.agents/directives/06-session-coordination.md#sha256:13a6dc2c19bfeef0de3feef6bdf78ebcba5226753a0a663520c63e8cfdf42913"
links: [source-development]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Temporary Git Worktree Lifecycle

Ordinary work uses one primary worktree. Create another only for a workload that needs parallel
independent changes and cannot safely run in sequence. Record its ownership and removal boundary.
Remove it immediately after its commits are reachable, verification passes, and no uncommitted or
unpushed required work remains. Force removal and primary-worktree removal are prohibited.
