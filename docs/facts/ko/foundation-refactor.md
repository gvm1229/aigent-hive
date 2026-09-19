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
  - "repo:docs/architecture/policy-rule-inventory.md#sha256:925d51703dc8db78bb15bc7598d18225766fae6aeb582fa31bd2c2a0ad58921d"
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:f811647803b6816a30e62ad03653f54700b7e53a4a876ec2802f83fde8332ca3"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:a1b96c88b4938c8771ccf635ffef0e8dcafa9ff1"
status: active
---

# 0.11.0 기반 구조 리팩터링 범위

- 미출시 `0.10.4` 계승, 목표 `0.11.0-test.1`; 별도 `0.10.4` 출시 제외
- 기존 기능·명령 유지, 지식 흐름 개선과 Codex 우선 검증
- 승인 7개: 실패 분류·결과 합산·실제 진단·보류 평가·고정 문맥 재사용·개선 후보 검토·문맥 유효성
- 후보 검토 `HK-005`, 나머지 기존 `HK-*`·`RFK-002` 소유
- 소스 브랜치 검사·파일 훅 변환·명시적 등록·실행 후보 검토 구현
- 실제 호스트 수용 미완료, 안정판·사용자 설치 별도 승인
