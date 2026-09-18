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
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:4052a4c14e26af16fefcd94116c8060f351ccb0fe5fe8321d127c59ff0d517a7"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:568f9540dc893c089dbd72244d7b0c0608518eeb"
status: active
---

# 0.11.0 기반 구조 리팩터링 범위

- 미출시 `0.10.4` 계승, 목표 `0.11.0-test.1`; 별도 `0.10.4` 출시 제외
- 기존 기능·명령 유지, 핵심 지식 흐름 개선과 Codex 우선 검증
- 소스 브랜치 검사 구현 완료
- 승인 제안 7개: 실패 분류·결과 합산·실제 진단·보류 평가·고정 문맥 재사용·개선 후보 검토·문맥 유효성
- 후보 검토는 `HK-005`, 나머지는 기존 `HK-*`·`RF-K02` 소유
- 제품 훅·실제 수용은 후속 작업, 안정판은 버전별 명시 승인 필요
