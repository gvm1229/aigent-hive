---
schema_version: 1
pair_id: ship-commit-granularity
topic_slug: ship-commit-granularity
language: ko
counterpart: ../en/ship-commit-granularity.md
title: "Ship commit 분리"
summary: "제품 ship Skill의 전체 변경 범위 처리와 독립 검토·되돌리기 가능한 관심사별 commit 분리 계약."
tags: [commit, git, skill, workflow]
aliases: ["Atomic ship commits"]
sources:
  - "repo:docs/skills.md#sha256:9d445726c92856de8c47781743cfc972aeeebbd1e74cc1660b860dd0ebac573a"
links: [public-skill-identity, source-development]
reviewed_revision: "git:23dafb9d646ea893ce06f6ec2cc9ea22b7eed673"
status: active
---

# Ship commit 분리

제품 `ship` Skill: 저장소 Git 규칙 확인, 전체 worktree 점검, 독립 검토·되돌리기 가능한 관심사별
파일 또는 hunk·가까운 검증·예정 commit의 사전 concern map 작성. 모든 파일 또는 모든 변경 요청:
전체 관심사 범위 권한, 단일 aggregate commit 권한 아님.

관심사별 stage·검증·commit 순차 처리, commit 뒤 concern map 갱신, 한 파일의 복수 관심사에 대한
patch stage 적용, 소유권 또는 범위 불명확 변경 미처리. 저장소 hook·이력·명시적 push 경계 보존.
