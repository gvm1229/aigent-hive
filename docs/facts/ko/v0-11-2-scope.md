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
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:72f2989faf99a78eb09b3f84fe6feb8842e3004b575c7267e4048a2213ee3214"
links: [knowledge-storage, marketing-deck-record, product-purpose]
reviewed_revision: "git:1f642217752c4d6bb7fa85d1d296ca1fa66968cf"
status: active
---

# 0.12.0 범위와 Windows 우선 순서

- 2026-10-05 사용자 지정 0.12.0 전환: 기존 0.11.2의 사용량·중단·호스트 소유 분담·Graphify 잔여·진단 범위 유지
- 검증된 기능만 활성, 나머지 비활성·대기. 활성 관계 기능은 저장 뒤 자동 분석, 실패도 저장 성공 보존
- Windows 우선·macOS 마지막. Claude 실제 검증·서명 대기
- Notion·발표 폐기, 로고·벡터 완료, 엔진 비교 보존. Obsidian 별도 플러그인 제외
- 조사·구현과 실제 호스트 수용 증명 구분
- 추가 0% 보호 기본 활성·별도 대화 한정 제외. 새 Hive 작업 차단, 진행 중 호스트 중단은 미검증
- 2026-10-06 자동 수용의 상태 표시 변경·Windows 공식 복구에 필요한 수동 승인 허용
