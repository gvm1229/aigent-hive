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
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:3e395404715ede791de670330489065939b0c81292a7897e30cdfacae601085c"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:5829f07a1626ec8adad1aad8bdbfd0ec25bc0fd6"
status: active
---

# 0.11.0 기반 구조 리팩터링 범위

- 유지보수자 선택: 미출시 `0.10.4` 변경을 계승하는 `0.11.0-test.1`, 별도 `0.10.4` 출시 제외
- 기존 기능·명령 유지, 지식 저장·조회·새 대화 연결부터 개선
- 실제 검증: Codex 우선, 이후 다른 호스트 확대
- 현재 승인 범위: 브랜치와 구현 계획. 제품 구현·실제 수용은 후속 작업
- 안정판: 해당 버전의 별도 명시 승인 필요
