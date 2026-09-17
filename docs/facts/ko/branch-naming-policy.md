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
  - "repo:docs/research/branch-naming-audit-2026-09-18.md#sha256:c84ae3cfef5fea40e68e6491930c2b53952eb9211b7f4d0f1b7f21deb785abd8"
links: [git-worktree-lifecycle]
reviewed_revision: "git:f580d1e7cbd2727d02cead3c42af66cd244802c5"
status: active
---

# 소스 브랜치 이름 규칙

- 유지보수자 요구: `main`·`develop` 외 브랜치에 `feature/`, `fix/`, `release/`, `docs/`, `test/`, `refactor/`, `build/`, `chore/` 접두사 필수
- `codex/` 등 에이전트 이름 접두사 금지, 작업 브랜치 생성의 명시 승인 유지
- Git 참조 문법 검사와 프로젝트 이름 정책 검사의 구분
- 기존 `staging` 예외 정합화와 강제 검사 추가는 조사 보고서의 제안; 구현 완료 주장 제외
