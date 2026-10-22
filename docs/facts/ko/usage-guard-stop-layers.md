---
schema_version: 1
pair_id: usage-guard-stop-layers
topic_slug: usage-guard-stop-layers
language: ko
counterpart: ../en/usage-guard-stop-layers.md
title: "사용량 보호의 중단 계층"
summary: "0.13.2 소스의 통신 호환 검사·현재 Windows의 초기화와 하한 중단 수용"
tags: [guard, history, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:00874e59d2c865710877e9d224617d7b5d736d76c6f597703ec65b8268261992"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:15d7691b39f9d6e160c2857681d2715ba59ece081f832f9da4df7942ffab6cdf"
  - "repo:crates/hive-cli/src/usage_monitor/threshold.rs#sha256:9c3f5e81cd6ddf7c4177448f63c31f49054bee8b4335a403985d9bbebf85b275"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/research/stable-release-0.13.1.md#sha256:8bb27b7af203a2a4c6a54dc39aa7fd978075e1fb373e6b6d9617e8550acaef4e"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:6096c88095e0ba39fc78dfe959356c2f8d988c2115e3b5519bd55ccf3041464d"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:332849449782b95d9d5819316151e0e8991477fa2cc56ba5df568dcc35f4c237"
  - "repo:tests/results/release-0.13.1-stable.md#sha256:a8c2566122cb825217755f8c9901570e63517e114fbe7a484a7ce6af229e1420"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:ae9ad865350a8e411d89a3bf77d7e6a8433d78302247b7bba51fdde444e918a9"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# 사용량 보호의 중단 계층

0.13.2 소스의 앱 신원·통신 응답 검사, 정확 버전 목록 제거. Windows·Codex26.1016.8120.0 합성 관측의 실제 중단:5%664ms·초기화1,078ms·0%1,161ms·0% 새 실행758ms. 앱 유지·일반 하한/초기화 뒤 직접 재개 확인. 연결·관측·중단의 별도 판정. 실제 계정 초기화는 설치0.13.1의 감지만 확인, 제어 실패. 수정본의 자연 초기화·이번 AI 하위 작업·과금 종료는 미검증. 일반 외부 명령 잔존 가능, 이전 계층·실패 근거 보존.
