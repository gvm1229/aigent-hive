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
  - "repo:.agents/directives/03-workflow.md#sha256:3ed250c39a40765032e18cf624c72441741476cc33fa797c2765d24a222fe14f"
  - "repo:.agents/directives/06-session-coordination.md#sha256:a24536201b77619549620d88612c186b769e90a774043895370a064779d8d758"
links: [source-development]
reviewed_revision: "git:f6139fe4aabe5237bb1da5cb85364da7c978e698"
status: active
---

# Temporary Git Worktree Lifecycle

Ordinary work uses one primary worktree. Create another only for a workload that needs parallel
independent changes and cannot safely run in sequence. Record its ownership and removal boundary.
Remove it immediately after its commits are reachable, verification passes, and no uncommitted or
unpushed required work remains. Force removal and primary-worktree removal are prohibited.
