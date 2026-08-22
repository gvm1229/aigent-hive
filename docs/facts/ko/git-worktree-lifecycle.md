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
  - "repo:.agents/directives/03-workflow.md#sha256:3ed250c39a40765032e18cf624c72441741476cc33fa797c2765d24a222fe14f"
  - "repo:.agents/directives/06-session-coordination.md#sha256:a24536201b77619549620d88612c186b769e90a774043895370a064779d8d758"
links: [source-development]
reviewed_revision: "git:f6139fe4aabe5237bb1da5cb85364da7c978e698"
status: active
---

# 임시 Git 작업 폴더 수명주기

일반 작업: 단일 기준 작업 폴더 우선. 안전한 순차 처리 불가한 병렬 독립 변경 때만 추가 작업 폴더 생성.
소유·제거 경계 기록. commit 도달성·검증·미반영 작업 없음 확인 뒤 즉시 제거. 강제 제거·기준 작업 폴더 제거 금지.
