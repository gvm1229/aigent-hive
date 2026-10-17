---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "잠금 없는 한 차례 중단·직접 재개·Windows 검증과 로컬 감시"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:fb83162490270dc48650dd60c0a93c321c8811915d863fd102e478f9d22967f4"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:39d24941f041d49e4aaee3c9130d0e430a7a14a5328557701e223dc93299aa65"
  - "repo:crates/hive-cli/src/usage_monitor/desktop.ps1#sha256:4c283db89066f295738ebc39b8da71faed536723e630170c4f67466412a5309f"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:7c60015123b18b54f4b2a1984839d5a3e5fc96b4935b8ded0ba7a6367d41279d"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:3a1337362f351951f06db51fc89b8fa06eea5b458534a06a2dc45463c7766139"
  - "repo:docs/guides/installed-usage-guard.md#sha256:369a6681fcc7415c711ab7d5b9d1dae879b9d6fc188261af10a4ffcb5327a6f2"
  - "repo:docs/research/release-0.13.0.md#sha256:2179351916477c5606006a5c08cf69d99e9155a80d5abca87acb69d45adbe5cf"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:50b5ba7baa1b9c2ed7bf96669115860655bb2fe0b7b5d7df7ac5fa628c650437"
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:58148052d858dec443c5d8231c0ad91c7748dfd8e54e2a5b5939e6fc5790cffd"
  - "repo:tests/results/session-stop-one-shot-installation-0.12.1.json#sha256:bbc233760b14fc02ebf35467618df2e59c26bab4e549806634a3c96d058d2371"
  - "repo:tests/results/session-stop-one-shot-qualification-0.12.1.json#sha256:000da296ad912dc0d30d30baed0f79f8a88193d6b09606bd75eff29e1dbe4406"
  - "repo:tests/results/session-stop-partial-windows-0.12.1.json#sha256:1f1d27dae9ef95f345c39fa018dce9f01b752cd765d24de3609b1ac7280a4071"
links: [automatic-dispatch-guard, installed-usage-guard, usage-stop-notice]
reviewed_revision: "git:6cee0fc8998d4b1b25eaed092f1a252b7144b210"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 0.13.0 공개 목표의 선택형 감시: 실제 초기화 또는 정상 조회 공백 60초 때 한 차례 중단, 15초 시작·5초 제한·2초 재시도
- 재개 잠금·확인 명령 없음. Codex 직접 재개·새 입력 허용, 같은 사건·재시작·시작 전 검사의 중복 중단 억제
- 0.12.1-dev Windows의 이전 네 실행 실제 중단 283~306ms·앱 유지·현재 읽기 4/4. 최종 한 차례 제어의 모의·Rust 회귀와 로컬 설치·정상 감시 확인
- 하한·0% 보호 유지. 다른 OS·클라우드·실제 제공자 초기화·서버 과금 종료 미증명
- 같은 중단·알림 정본 코드로0.13.0 소스 검사 통과, 공개 패키지 수용과 구분
