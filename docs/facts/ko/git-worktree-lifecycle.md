---
schema_version: 1
pair_id: git-worktree-lifecycle
topic_slug: git-worktree-lifecycle
language: ko
counterpart: ../en/git-worktree-lifecycle.md
title: "임시 Git 작업 폴더 수명주기"
summary: "관리자 세션의 필요한 임시 작업 폴더 생성 허용, 검증 뒤 소유 폴더·병합 브랜치 정리"
tags: [git, workflow, worktree]
aliases: ["temporary clone cleanup", "worktree cleanup"]
sources:
  - "repo:.agents/directives/06-session-coordination.md#sha256:8c889202b65d024691c567253833d3d57f621800c0802ab83d1bdf0df1c0cd20"
  - "repo:.agents/directives/references/git-worktrees.md#sha256:74afce43b00e5e72b0342020cc4022446aee632b0d10f615a5509e31ed41ea7f"
links: [source-development]
reviewed_revision: "git:e032bd9f8a1e404cf473dfd78982c9f7780abb13"
status: active
---

# 임시 Git 작업 폴더 수명주기

- 관리자 세션: 병렬·격리 작업에 필요한 수만큼 임시 작업 폴더 생성 허용(관리자, 2026-11-12)
- 생성 전 소유자·제거 경계 기록
- 병합·종료 뒤 커밋 도달성과 필요한 push 확인, 소유 임시 폴더 제거, 로컬·원격 병합 브랜치 삭제, prune
- 종료 시 소유 임시 작업 폴더 잔존 금지, 미커밋·미병합 작업의 안전한 보존
- 강제 제거·기준 작업 폴더 제거 금지
