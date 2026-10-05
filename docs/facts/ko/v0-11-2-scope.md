---
schema_version: 1
pair_id: v0-11-2-scope
topic_slug: v0-11-2-scope
language: ko
counterpart: ../en/v0-11-2-scope.md
title: "0.12.0 범위와 Windows 우선 순서"
summary: "사용자 결정에 따른 0.12.0 범위, Windows 우선과 macOS 최종 수용."
tags: [scope, v0-12-0, windows]
aliases: []
sources:
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:84a2fba7eacd840fdd0bac536595b58b6f73afb2381fea7c8cc971a3ae1f6247"
links: [knowledge-storage, marketing-deck-record, product-purpose]
reviewed_revision: "git:ad412a7b2aa6bba927f5f841e0e5161f4fb60488"
status: active
---

# 0.12.0 범위와 Windows 우선 순서

- 2026-10-05 사용자 지정 0.12.0 전환: 기존 0.11.2의 사용량·중단·호스트 소유 분담·Graphify 잔여·진단 범위 유지
- 검증된 기능만 활성, 나머지 비활성·대기. 활성 관계 기능은 저장 뒤 자동 분석, 실패도 저장 성공 보존
- Windows 우선·macOS 마지막. Claude 실제 검증·서명 대기
- Notion·발표 폐기, 로고·벡터 완료, 엔진 비교 보존. Obsidian 별도 플러그인 제외
- 조사·구현과 실제 호스트 수용 증명 구분
- 추가 0% 보호 기본 활성·별도 대화 한정 제외. 새 Hive 작업 차단, 진행 중 호스트 중단은 미검증
