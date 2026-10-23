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
  - "repo:docs/research/public-test-0.13.2.md#sha256:138f0f7939a701b13860e9aeaa3c32f85e2723387c4a183d40c5473e7d58435e"
  - "repo:docs/research/stable-release-0.13.1.md#sha256:8bb27b7af203a2a4c6a54dc39aa7fd978075e1fb373e6b6d9617e8550acaef4e"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:44f1fb5716b0af3ab556a6d03c38c10d3422a1dc0b2489b79be106f43a62e862"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:docs/research/usage-guard-stop-comparison.md#sha256:bdd17b2745522c0f5ed813a4dcf1673e0fa34185970bca7e20402f5eac55015f"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:f901c85a4769dba3021471341fe9a14c0daba392661212bd233d852bfed80304"
  - "repo:tests/results/public-test-0.13.2-test.2.json#sha256:7eb3e11914f44421bbaa13db06d2b3edfc2c791170586eee259355ec75587005"
  - "repo:tests/results/release-0.13.1-stable.md#sha256:a8c2566122cb825217755f8c9901570e63517e114fbe7a484a7ce6af229e1420"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:76abaa5e2595b9aeee3797fd612694112e9c8efbbcb43ccfaed49653907aeafc"
links: [automatic-dispatch-guard, installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:5a528fd98f36697d2692436569dbe56ec3888bbf"
status: active
---

# 사용량 보호의 중단 계층

0.13.2-test.2의 앱 신원·응답 검사, 번호 제한 제거. Windows 11 x64·Codex26.1016.8120.0 개발 파일의 시험값 실제 중단:5%664ms·초기화1,078ms·0%1,161ms·새0%758ms. 앱 유지·일반 하한/초기화 뒤 직접 재개, 연결·관측·중단 별도. 공개 제어 코드 일치. 공개 파일의0.13.1 미완료 갱신 복구·자료 보존·정상4+4조회 확인. 승인 초기화 근거는 실제0→100·동일 감지·실제 중단 결합. 당시0.13.1 제어 실패, 수정본 자연 초기화·이번AI하위 작업·과금 종료 미검증. 일반 외부 명령 잔존, 과거 근거 보존.
