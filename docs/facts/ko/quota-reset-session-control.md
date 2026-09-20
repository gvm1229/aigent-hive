---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "초기화 감지의 작업별 제외와 잔여량 하한·기존 초기화 중단 보호의 분리"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:c94975f1e11052ebf9c04e00066fe121229eea186945c056164d3a33c609df87"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:72bd93df896956e10c30013db3bc9232b7a4a3ca"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 미완료 0.11.0 항목 해결 요청에 따른 `disable-reset-guard`·`enable-reset-guard` 추가
- 제외 전용 확인과 정확한 호스트·세션·프로세스 결합
- 전역 설정·잔여량 하한·측정 불가 차단 유지, 기존 초기화 중단 해제 금지
- 두 동작 뒤 새 `enforce` 필수, 다른 연결로 제외 권한 이전 금지
- Windows CLI 회귀로 위 경계 검증. 주기 감시·진행 중 추론 중단은 미검증
