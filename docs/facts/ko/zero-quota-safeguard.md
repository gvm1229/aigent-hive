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
  - "repo:docs/guides/installed-usage-guard.md#sha256:735fffd7960d1dca621b1de7bb2fa7f77ef232cda6c75c4165afda76c9f8da50"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:6096c88095e0ba39fc78dfe959356c2f8d988c2115e3b5519bd55ccf3041464d"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:332849449782b95d9d5819316151e0e8991477fa2cc56ba5df568dcc35f4c237"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:ae9ad865350a8e411d89a3bf77d7e6a8433d78302247b7bba51fdde444e918a9"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# 기본 활성 0% 소진 보호

- 요청: 크레딧 자동 사용 위험을 줄이는 기본 활성 보호
- 개발0.12.0: 일반 보호 해제 뒤에도 관측된 구독 창·모음 하나라도0이면 새 Hive 작업 차단. 누락·손상·만료도 차단
- 별도 확인의 제외: 정확한 현재 호스트·대화·프로세스만 적용, 다른 결합 기본 활성
- 단순 재개·초기화 확인으로 소진 해제 불가. 새 양수 관측의 해제와 초기화 보호의 별도 확인 구분
- Windows 공개0.12.0-test.2 합성28개 통과·POSIX 링크1개 제외. 실제 유료 전환 방지·진행 중 호스트 중단·유료 설정 변경 미증명
- 해당 시험 당시 사용자 설치0.11.1 보존, 현재 설치와 구분

0.13.2 소스의 하한0~99% 지원.0은 보호 활성 상태, 양수 할당량의 실행 허용과 크레딧 잔액과 무관한 소진 차단. 소진 중단 기록의 실행별 구분으로 새 실행의 이전 중단 기록 승계 방지. Windows·Codex26.1016.8120.0의 합성 관측 시험에서 실제 도구 실행 중단1,161ms·새 실행 중단758ms·앱 생존 확인. 소진 확인 뒤 모델의 추가 도구 호출 금지. 실제 제공자 초기화 시 중단·이번 AI 하위 작업 중단·과금 손실0은 미검증.
