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
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:09a27cf643e59088c6966d4a10e659069662860c8646db86b195f5bd1d13bafb"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:798f0e43b17f3446547899b0355cca2b177779889b1245ce4cec7dd0bfcc26de"
  - "repo:crates/hive-cli/src/usage_monitor/threshold.rs#sha256:9f00da30cddf8bbc0083c2e90786e3061fe50947cf41e183c4c8ee9bd91af5c1"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:a5e5881578cd99f1583a00c5a14bca9bc1324a48"
status: active
---

# 사용량 보호의 중단 계층

- 0.7~0.9 공개 보호: 새 자동 실행 차단
- 소스 전용15초 감시: halt.json·다음 검사, 추론 직접 취소 제외.2026년8월 제거
- 선택형0.13.0: 관측·초기화 자료와 Codex 직접 중단, 초기화·60초 공백 조건
- 0.13.1: 기존 하한 공유·소유 대화별 재개 기록
- Windows 추론·AI 하위 작업·도구 호출 중 대화 중단·원인 안내·동일 하한 재개 확인
- 일반 외부 명령의 잔존 허용, AI 하위 작업 중단 필수. 과거 셸 종료 실패 보존
- 과거 추론 중단·다른 호스트·과금 동시 종료의 증명 제외
