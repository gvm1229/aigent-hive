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
  - "repo:.agents/directives/06-session-coordination.md#sha256:c0659539029a3cdaf36241b2f67c6aefebba5757745f69996cf72e2b52aa0791"
  - "repo:.agents/directives/references/git-worktrees.md#sha256:c005aaa111e2ec2376f9cdd914fa1b215163f9f5f5a0ef0313f0709a69c24252"
links: [source-development]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Temporary Git Worktree Lifecycle

Ordinary work uses one primary worktree. Create another only for a workload that needs parallel
independent changes and cannot safely run in sequence. Record its ownership and removal boundary.
Remove it immediately after its commits are reachable, verification passes, and no uncommitted or
unpushed required work remains. Force removal and primary-worktree removal are prohibited.
