---
schema_version: 1
pair_id: usage-guard-stop-layers
topic_slug: usage-guard-stop-layers
language: ko
counterpart: ../en/usage-guard-stop-layers.md
title: "사용량 보호의 중단 계층"
summary: "과거 협력형 검사와 현재 직접 실행 취소의 보장 범위 구분."
tags: [guard, history, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:39d24941f041d49e4aaee3c9130d0e430a7a14a5328557701e223dc93299aa65"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:7c60015123b18b54f4b2a1984839d5a3e5fc96b4935b8ded0ba7a6367d41279d"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:11435eda675af021821e84d452ade72efdc18b2e3829aa67ac7af88ecdcc09bd"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:914c06a3a071de37d529ab6ca4626f5cf548e129"
status: active
---

# 사용량 보호의 중단 계층

- 사용자 질문:0.7~0.9 한도 보호와 새 중단의 차이·재사용·난항 원인
- 기존 공개 보호: 새 자동 실행 차단. 소스 전용15초 감시: halt.json 기록과 에이전트의 다음 검사에서 중단, 추론 중 직접 취소 아님.2026년8월 제거
-0.13 선택형 감시: 사용량 수집·초기화 자료 재사용, 독립 감시와 Codex 직접 중단 추가
- 직접 중단 조건은 초기화·60초 관측 공백. 일반 잔여량 하한은 기존 사전 검사에 남아 직접 취소와 미연결
- 과거 중단 경험은 다음 검사에서 멈춘 결과와 양립. 특정 과거 실행·다른 호스트의 즉시 중단 증명 없음
