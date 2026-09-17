---
schema_version: 1
pair_id: branch-naming-policy
topic_slug: branch-naming-policy
language: ko
counterpart: ../en/branch-naming-policy.md
title: "소스 브랜치 이름 규칙"
summary: "main·develop 외 소스 브랜치의 작업 종류 접두사 의무와 에이전트 이름 접두사 금지"
tags: [branch, git, policy]
aliases: []
sources:
  - "repo:docs/guides/branching-rules.md#sha256:3820e9a509c4eaee972e2900550d03d2f1a8f7140c8dbc3f6adf1c55596a00ae"
  - "repo:scripts/branch-policy.py#sha256:3be4d5412e7c4060636f93f612b1df9b118ff9fa54d980e805a6362fdbb3f1f7"
links: [git-worktree-lifecycle]
reviewed_revision: "git:a642bd91e1cabdf80673b836f2e801a2c6cf9b61"
status: active
---

# 소스 브랜치 이름 규칙

- 허용: `main`·`develop` 또는 `feature/`, `fix/`, `release/`, `docs/`, `test/`, `refactor/`, `build/`, `chore/` 작업 브랜치
- 생성의 명시 승인 유지, 단일 정책의 문법·생성·이름 변경·커밋·게시 목적지·PR 검사와 원격 제한 생성
- 로컬 Git 훅과 서버의 이름 제한 적용
- Git 2.45 직접 이름 변경의 참조 훅 우회 가능성: `rename` 도구와 커밋·게시 검사 병행
