---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "앱 유지·세션 중단·원인 표시와 정확한 사건 확인의 경계"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:5dd3088b00bc64fb53e3ad3be195b6b967e9b8e45433e4d915765b6626dd2d94"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:eea2f5a40cb2afa52e7acc6ed070b409d1f28f7b7bde3d545bce07baf9e8ab52"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b991ea10a2319c71ab62dea0c9ad3442dea2779d92e2d1bc215df1478077c6b4"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:a358f845a8f2a688af4b1dd5d081936d3673f960f1f6b37729cee01c75419a07"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:156c8cf863043b6e910c4b6bfa2dbb633382da49"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 0.12.1 Windows 제어 미검증·비활성. enable·run과 직접 PowerShell 통신 시작 전 거부, 앱 전체 종료 지원 없음
- 설치 CLI 0.12.0 원복, 감시 예약·자동 설치 해제. 기존 대화별 제외·하한·0% 보호 유지
- 구현 계약: 15초 시작·5초 제한·2초 재시도·30초 공백 중단과 사건 잠금, 정확한 확인·새 관측 필수. 자동 재개 금지
- 합성 신호의 부모·하위 작업 중단 부분 근거, 전체 실제 수용 실패. 앱 종료의 인과관계 미확정
- 안전한 세션 중단·상태 표시·설치 수용 미완료
