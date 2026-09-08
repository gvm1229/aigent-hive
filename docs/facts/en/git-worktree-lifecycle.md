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
  - "repo:.agents/directives/03-workflow.md#sha256:8d3afcb2e885232dcb7e7775d55d0b48477358ddbc0266ff4a48de80af34e9fc"
  - "repo:.agents/directives/06-session-coordination.md#sha256:af121dbbd4cc3f3d8141ef6ff10d5645c13f833e8f809609495a249079b92424"
links: [source-development]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Temporary Git Worktree Lifecycle

Ordinary work uses one primary worktree. Create another only for a workload that needs parallel
independent changes and cannot safely run in sequence. Record its ownership and removal boundary.
Remove it immediately after its commits are reachable, verification passes, and no uncommitted or
unpushed required work remains. Force removal and primary-worktree removal are prohibited.
