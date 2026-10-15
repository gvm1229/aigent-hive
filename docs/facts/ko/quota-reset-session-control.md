---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "첫 차수의 실제 초기화·60초 공백 중단, 현재 앱 제어 비활성 유지"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:87fcbc7b005837f38639ceef43113c232ed5742298886caea92c009c9921d5cd"
  - "repo:crates/hive-cli/src/usage_monitor/desktop.ps1#sha256:ac6b2bafb909ef4afc591b16555346a15004b318fc6439de559e928f3d53ee99"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:91ab7f3d580cbc6f69a945072be09820c321c58ebcb247df033c5df88d85854b"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:c45862bad77bd8cb0b9eb04691cf6161074509e7afeed1effb21cc57b03f532f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:c51194775aaf8b9f4b0c34ffabdbbdd09768fb0758bdd54251d6f3d1a6f259b0"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:5035226eda596f7241ac9f730af31a78c72f61d2804b137849db1a572d7e1f47"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:7673c0ca72551d413d7f9e41f00d88c3f388b027"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 0.12.1 첫 차수: 세션 중단만 포함. 실제 초기화 즉시 요청·정상 조회 공백 60초 보호
- 15초 시작·5초 제한·2초 재시도. 평상시 기록 구독 없음, 소유 대화 중단 뒤 5초 확인
- 앱 종료 금지, 공유 사건의 정확한 확인·새 관측 필수. 대화 제외·재시작으로 잠금 해제 불가, 자동 재개 금지
- 하한·0% 보호 유지. Windows 모의 통과, 현재 앱 수용 미완료·활성 차단·설치 CLI 0.12.0
- 이전 부모·하위 작업 중단은 부분 근거, 앱 종료 원인 미확정
