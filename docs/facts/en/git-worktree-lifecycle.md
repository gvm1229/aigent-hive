---
schema_version: 1
pair_id: git-worktree-lifecycle
topic_slug: git-worktree-lifecycle
language: en
counterpart: ../ko/git-worktree-lifecycle.md
title: "Temporary Git Worktree Lifecycle"
summary: "Authorized temporary worktrees and clean clones require recorded ownership and task-bound cleanup."
tags: [git, workflow, worktree]
aliases: ["temporary clone cleanup", "worktree cleanup"]
sources:
  - "repo:.agents/directives/03-workflow.md#sha256:80eaeeb1cf3452f735332b8acdbe2814b6efed94370476dd665fe9554f23642e"
  - "repo:.agents/directives/06-session-coordination.md#sha256:9d7f2ecfc3a3fc2226c3ffe5f2778c3f538a73ee8a49ea7028d43c6a644eacdc"
links: [source-development]
reviewed_revision: "git:02e8bfc95913b1d88c4324dbb997d19fc55ef767"
status: active
---

# Temporary Git Worktree Lifecycle

An authorized temporary worktree or clean-context clone records its absolute path, ref, purpose,
owner, and removal boundary in the active-session manifest. After its concern is committed,
pushed, and verified, its owner removes a clean worktree, prunes the registry, and verifies the
registry before the final response. A dirty path must be committed and pushed when authorized, or
be retained with its exact reason. Force removal and primary-worktree removal are prohibited.
