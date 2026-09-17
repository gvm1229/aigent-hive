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
  - "repo:docs/guides/branching-rules.md#sha256:76be6ab2fa1ad493840d33974d9b6b30c8b89daa0619f2cdc7f7f21eceda745b"
  - "repo:scripts/branch-policy.py#sha256:e1dd087bf7d2bf9f4057a89959e89690a69486cd05b42fc1ad87c7ee2134f394"
links: [git-worktree-lifecycle]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# 소스 브랜치 이름 규칙

- 허용: `main`·`develop` 또는 `feature/`, `fix/`, `release/`, `docs/`, `test/`, `refactor/`, `build/`, `chore/` 작업 브랜치
- 생성의 명시 승인 유지, 단일 정책의 문법·생성·이름 변경·커밋·게시 목적지·PR 검사와 원격 제한 생성
- 로컬 Git 훅과 서버의 이름 제한 적용
- Git 2.45 직접 이름 변경의 참조 훅 우회 가능성: `rename` 도구와 커밋·게시 검사 병행
