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
  - "repo:docs/architecture/policy-rule-inventory.md#sha256:77bd57357ba6cb08f84c1ee8908d8859514bd114951faf6abc92b67d7ef30920"
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:894919913703c2959487ce2699a10b339a9934945f05240e997f076143ed738e"
links: [artifact-boundaries, knowledge-source-freshness, orchestration-ownership]
reviewed_revision: "git:dd4e2773e98d7acb13fa0dc77ad11167faaf23fb"
status: active
---

# 0.11.0 기반 구조 리팩터링 범위

- 미출시 `0.10.4` 계승, 목표 `0.11.0-test.1`; 별도 `0.10.4` 출시 제외
- 기능·명령 유지, 지식 흐름 개선·Codex 우선 검증
- 승인된 호환성 예외: 자동 재개의 현재 세션·양수 프로세스 ID 필수
- 승인 7개: 실패 분류·결과 합산·실제 진단·보류 평가·고정 문맥 재사용·개선 후보 검토·문맥 유효성
- 후보 검토 `HK-005`, 나머지 기존 `HK-*`·`RFK-002` 소유
- 소스 브랜치 검사·파일 훅 변환·명시적 등록·실행 후보 검토 구현
- 실제 수용: Codex·Antigravity, Claude·상시 감시·자동 중단은 후속 목표; 안정판·사용자 설치 별도 승인
