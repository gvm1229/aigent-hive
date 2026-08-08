---
schema_version: 1
pair_id: git-worktree-lifecycle
topic_slug: git-worktree-lifecycle
language: ko
counterpart: ../en/git-worktree-lifecycle.md
title: "Temporary Git worktree lifecycle"
summary: "허가된 temporary worktree·clean clone의 소유 기록과 task-bound cleanup."
tags: [git, workflow, worktree]
aliases: ["temporary clone cleanup", "worktree cleanup"]
sources:
  - "repo:.agents/directives/03-workflow.md#sha256:80eaeeb1cf3452f735332b8acdbe2814b6efed94370476dd665fe9554f23642e"
  - "repo:.agents/directives/06-session-coordination.md#sha256:9d7f2ecfc3a3fc2226c3ffe5f2778c3f538a73ee8a49ea7028d43c6a644eacdc"
links: [source-development]
reviewed_revision: "git:02e8bfc95913b1d88c4324dbb997d19fc55ef767"
status: active
---

# Temporary Git worktree lifecycle

허가된 temporary worktree·clean-context clone: active-session manifest의 absolute path·ref·purpose·owner·removal boundary 기록.
Concern commit·push·verification 뒤 clean worktree 제거, registry prune, final response 전 registry 확인.
Dirty path: 허가된 commit·push 또는 exact retained reason. Force removal·primary worktree removal 금지.
