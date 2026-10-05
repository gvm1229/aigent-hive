---
schema_version: 1
pair_id: v0-11-2-scope
topic_slug: v0-11-2-scope
language: ko
counterpart: ../en/v0-11-2-scope.md
title: "0.11.2 범위와 Windows 우선 순서"
summary: "사용자 결정에 따른 0.11.2 범위, Windows 우선과 macOS 최종 수용."
tags: [scope, v0-11-2, windows]
aliases: []
sources:
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:559da92582358c331cfd939bfeb380f31c3361f3e92ba8cb2294b013f05a8fe3"
links: [knowledge-storage, marketing-deck-record, product-purpose]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# 0.11.2 범위와 Windows 우선 순서

- 2026-10-05 결정: 사용량 감시·중단, 호스트 소유 분담, Graphify 잔여·진단의 0.11.2 편입
- 검증된 기능만 활성, 미지원·미검증 호스트는 비활성·대기. 활성 관계 기능은 저장 뒤 자동 분석, 분석 실패도 저장 성공 보존
- Windows 우선, macOS 전용 수용 마지막. Claude 실제 검증·게시자 서명 대기
- Notion·오래된 발표 자료 폐기. 로고·벡터 완료, 엔진 비교 보존. Obsidian은 Markdown 사용, 별도 플러그인 제외
- 공식 자료·CLI·소스 조사에 따른 계획. 조사·구현 완료와 실제 호스트 수용 증명 구분
