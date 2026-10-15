---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "작업별 초기화 제어와 Windows 앱 중단 감시의 별도 확인 경계"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:51544b81ad816dce429781e8ac48b3c2f6229a03f3feadfa4b2a78693eea93ca"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:f5bb2d7205520cb7abed21b156176e6aff1952ccab5858cbb9ece69299ca015c"
  - "repo:docs/guides/installed-usage-guard.md#sha256:f1a23a8779e4272fd852313360653f4078bcabd1290c4bebda9c8b0589d077fc"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:eb62d9db75f946f948b4dada89fa141654e80135"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 작업별 제외: 호스트·세션·프로세스 결합과 확인 필수. 하한·측정 불가 차단·기존 초기화 중단 유지, 변경 뒤 새 검사
- 0.12.1 Windows Codex 별도 감시기: 15초 공유 관측, 초기화 뒤 앱 전체 비상 종료 요청, 정상 관측 공백 30초 뒤 보호 중단·앱 밖 알림
- 새 채팅·재시작에도 사건과 첫 종료 확인 보존. 정확한 사건 확인과 새 관측 뒤 해제, 자동 작업 재개 없음
- 요청 목적: 초기화 전 사용 지속·초기화 뒤 새 할당량 보존. Windows 상태·프로세스 표본 검증, 실제 앱 수용 통과 뒤 설치의 별도 증명
