---
schema_version: 1
pair_id: git-worktree-lifecycle
topic_slug: git-worktree-lifecycle
language: ko
counterpart: ../en/git-worktree-lifecycle.md
title: "임시 Git 작업 폴더 수명주기"
summary: "일반 작업은 단일 기준 작업 폴더 우선, 임시 작업 폴더는 검증 뒤 즉시 정리"
tags: [git, workflow, worktree]
aliases: ["temporary clone cleanup", "worktree cleanup"]
sources:
  - "repo:.agents/directives/06-session-coordination.md#sha256:c0659539029a3cdaf36241b2f67c6aefebba5757745f69996cf72e2b52aa0791"
  - "repo:.agents/directives/references/git-worktrees.md#sha256:c005aaa111e2ec2376f9cdd914fa1b215163f9f5f5a0ef0313f0709a69c24252"
links: [source-development]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# 임시 Git 작업 폴더 수명주기

일반 작업: 단일 기준 작업 폴더 우선. 안전한 순차 처리 불가한 병렬 독립 변경 때만 추가 작업 폴더 생성.
소유·제거 경계 기록. commit 도달성·검증·미반영 작업 없음 확인 뒤 즉시 제거. 강제 제거·기준 작업 폴더 제거 금지.
