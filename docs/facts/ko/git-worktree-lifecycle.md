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
  - "repo:.agents/directives/03-workflow.md#sha256:a96876118609c4f6b116dc666493c31335b0389c6129462fdbc021afe1b1d9d6"
  - "repo:.agents/directives/06-session-coordination.md#sha256:884fedad85a6bd5c7865b5fc6be9b132c4653abb8d685f26aff621596f6ae48a"
links: [source-development]
reviewed_revision: "git:02e8bfc95913b1d88c4324dbb997d19fc55ef767"
status: active
---

# Temporary Git worktree lifecycle

허가된 temporary worktree·clean-context clone: active-session manifest의 absolute path·ref·purpose·owner·removal boundary 기록.
Concern commit·push·verification 뒤 clean worktree 제거, registry prune, final response 전 registry 확인.
Dirty path: 허가된 commit·push 또는 exact retained reason. Force removal·primary worktree removal 금지.
