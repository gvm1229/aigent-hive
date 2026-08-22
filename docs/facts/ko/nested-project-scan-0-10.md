---
schema_version: 1
pair_id: nested-project-scan-0-10
topic_slug: nested-project-scan-0-10
language: ko
counterpart: ../en/nested-project-scan-0-10.md
title: "Nested project scan 0.10.0 범위"
summary: "0.9.5의 0.9 release line 마감과 상위 Git repository 아래 registered project의 안전한 knowledge scan을 0.10.0으로 편입."
tags: [knowledge, release, scan, v0-10]
aliases: ["Nested Vault scan"]
sources:
  - "repo:docs/decisions/product-release-decisions.md#sha256:3fbe246c3a5b7d2b8ec002d40f73874c056c48ae3a888dede3e40db12eddddac"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:75ece4a12c890f3950d876c96ec605a2a80ebeecfc2ed7255ff3797cc2a33c2e"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
links: [knowledge-portability-scan, version-policy]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
status: active
---

# Nested project scan `0.10.0` 범위

- Release 결정: `0.9.5`가 마지막 `0.9.x`, `0.9.6` 게시 계획 없음
- `SCP10-003`: 상위 Git repository 아래 registered project root의 knowledge scan 복구
- 수락 경계: 등록 root 안 한정, sibling 접근·전역 Git 설정 변경 `0건`
- 탈출 방어: symlink·junction·reparse point 거부
