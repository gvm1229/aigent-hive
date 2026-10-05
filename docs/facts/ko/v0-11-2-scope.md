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
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:958df30cdb8c19d4591eed2324587cd0cdb3fb90a30da8ef01daee015efc150e"
  - "repo:docs/plans/backlog/0.12.0-deferred-host-acceptance.md#sha256:a08ecefcd819be4b39c1eaf2ee8707d026a0388b37a5c4ec0f5dd70ce4db6dde"
links: [knowledge-storage, marketing-deck-record, product-purpose]
reviewed_revision: "git:72e869a50b312d866ca4d5a826fe85b9a4afc414"
status: active
---

# 0.12.0 범위와 Windows 우선 순서

- 원래0.11.2의 사용량·중단·분담·지식 관계·계측·진단을0.12.0으로 전환
- 현재 사용자 결정: 완료 기능만의0.12.0 안정판 승인. 미완료9개는 연결된 후속 목록으로 이관, 목표 버전 미지정·완료 처리 제외
- Windows 우선·원래macOS 수용 마지막. 검증한 기능만 활성, 관계 분석은 저장 뒤 처리·저장 성공 보존
- 기본 활성0% 보호·명시적 대화 한정 제외. 진행 중 호출 중단과 별도인 새 Hive 작업 차단
- Claude·서명 대기, Notion·발표 폐기, 로고·벡터 완료·엔진 비교 보존, Obsidian 플러그인 제외
- 설명한 상태 표시 변경·복원과 공식Windows복구 승인. 소비자 설치는 별도 승인
