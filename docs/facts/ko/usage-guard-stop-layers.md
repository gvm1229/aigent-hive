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
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:700e67f2d621d40a9e718832d9b7d93c923f9ae996c36564014257f2b83f741b"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:fdbaac3fc1d9c6e8d5b52db8ca20d3e6d74de8bacc42cdd8919352c9d97512c7"
  - "repo:crates/hive-cli/src/usage_monitor/threshold.rs#sha256:9f00da30cddf8bbc0083c2e90786e3061fe50947cf41e183c4c8ee9bd91af5c1"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:724a2648f5a116878aa258996b42ee8810a8b6ec"
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
