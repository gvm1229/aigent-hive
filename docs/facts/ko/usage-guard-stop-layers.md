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
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:b8e7b8f90c305c715a2cd7a0e4e1fc89133ad57f8f1fbfe8aeef2e8efe62d20e"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:798f0e43b17f3446547899b0355cca2b177779889b1245ce4cec7dd0bfcc26de"
  - "repo:crates/hive-cli/src/usage_monitor/threshold.rs#sha256:9f00da30cddf8bbc0083c2e90786e3061fe50947cf41e183c4c8ee9bd91af5c1"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:20d7ea0fbdbfa3cb34bb0bd0548453660d556d628eda16a609e15a2e8ff562a9"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
status: active
---

# 사용량 보호의 중단 계층

-0.7~0.9 공개 보호는 새 자동 실행 차단. 소스 전용15초 감시는 halt.json과 다음 검사에서 중단, 추론 중 직접 취소 제외.2026년8월 제거
-0.13.0은 수집·초기화 자료 재사용, 독립 감시·Codex 직접 중단 추가. 조건: 초기화·60초 관측 공백
- 개발0.13.1은 기존 하한도 같은 경로에 연결, 재개 허용은 소유 대화에 결합
- 과거 검사 통과로 추론 중단 증명 불가. 새 긴 추론·도구 수용 대기, 다른 호스트 직접 중단 증명 제외
