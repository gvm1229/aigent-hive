---
schema_version: 1
pair_id: foundation-refactor
topic_slug: foundation-refactor
language: ko
counterpart: ../en/foundation-refactor.md
title: "0.11.0 기반 구조 리팩터링 범위"
summary: "미출시 0.10.4 변경을 계승하는 0.11.0 리팩터링, 기존 기능 유지와 Codex 우선 핵심 지식 흐름 검증"
tags: [architecture, refactor, version]
aliases: []
sources:
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:f548166391661d9103bf724f42c96bf073c9a4ba947103932b352806544ecfcd"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:23a94c06874fd65f8cdb0151f8512caed88fd792"
status: active
---

# 0.11.0 기반 구조 리팩터링 범위

- 유지보수자 선택: 미출시 `0.10.4` 변경을 계승하는 `0.11.0-test.1`, 별도 `0.10.4` 출시 제외
- 기존 기능·명령 유지, 지식 저장·조회·새 대화 연결부터 개선
- 실제 검증: Codex 우선, 이후 다른 호스트 확대
- 소스 브랜치 검사 구현 완료, 호스트 정책 훅의 리팩터링 계획 편입
- 제품 훅 구현·실제 호스트 수용은 후속 작업
- 안정판: 해당 버전의 별도 명시 승인 필요
