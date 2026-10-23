---
schema_version: 1
pair_id: zero-quota-safeguard
topic_slug: zero-quota-safeguard
language: ko
counterpart: ../en/zero-quota-safeguard.md
title: "기본 활성 0% 소진 보호"
summary: "0.13.2 소스의0% 하한·독립 소진 보호·Windows 실제 중단 수용"
tags: [exhaustion, session, usage, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6a8b7909e224bde9bcf45ea91e8267219b6a7b1a4546f3b601705db94d144a88"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:3c1f90928debba27a9bb5080e04d4db7b59b15aa32a5dd88b121eb10712624bf"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/guides/installed-usage-guard.md#sha256:0ead68f33595fcbd18d4d9c9ec38926645ca3cf22ffd895a02628c5e3c97a384"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:44f1fb5716b0af3ab556a6d03c38c10d3422a1dc0b2489b79be106f43a62e862"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:f901c85a4769dba3021471341fe9a14c0daba392661212bd233d852bfed80304"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:76abaa5e2595b9aeee3797fd612694112e9c8efbbcb43ccfaed49653907aeafc"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# 기본 활성 0% 소진 보호

0.13.2 소스의 하한0~99% 지원.0은 보호 활성, 양수 할당량 실행과 크레딧·일반 보호 해제에 무관한 소진 차단. 구간·모음 하나라도0이면 차단, 누락·만료·손상도 차단. 정확 호스트·대화·프로세스의 별도 승인만 제외. 새 양수 관측의 재검사 허용, 초기화 확인만으로 소진 해제 불가. 소진 기록의 실행별 구분과 새 실행 재차단. 소진 뒤 기록·검사·Git·정리·인계 도구 금지. 현재 Windows 합성 관측의 실제 중단 확인, 과금 방지·자연 초기화 중단은 미검증. 과거 수용 보존.
