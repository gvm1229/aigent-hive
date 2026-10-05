---
schema_version: 1
pair_id: zero-quota-safeguard
topic_slug: zero-quota-safeguard
language: ko
counterpart: ../en/zero-quota-safeguard.md
title: "기본 활성 0% 소진 보호"
summary: "일반 보호 해제 뒤에도 새 Hive 작업을 차단하는 0.12.0의 독립 소진 보호."
tags: [exhaustion, session, usage, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:41e5712b85c91de0f0df5505e999b245e60a530d5764f8cf4a35c285b8c4f730"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:2aaad6ac2375e212caa3bfc5d612f04e2d2c585e1a2560a294d8f4029613e566"
  - "repo:docs/guides/installed-usage-guard.md#sha256:916836c260db60424f19f6540ee5855e490e896f3681b658b07c8d1c063ddc05"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:16c01d6b233c96ee107914eeca730e3636c5440b"
status: active
---

# 기본 활성 0% 소진 보호

- 요청: 크레딧 자동 사용 위험을 줄이는 기본 활성 보호
- 개발0.12.0: 일반 보호 해제 뒤에도 관측된 구독 창·모음 하나라도0이면 새 Hive 작업 차단. 누락·손상·만료도 차단
- 별도 확인의 제외: 정확한 현재 호스트·대화·프로세스만 적용, 다른 결합 기본 활성
- 단순 재개·초기화 확인으로 소진 해제 불가. 새 양수 관측의 해제와 초기화 보호의 별도 확인 구분
- Windows 공개0.12.0-test.2 합성28개 통과·POSIX 링크1개 제외. 실제 유료 전환 방지·진행 중 호스트 중단·유료 설정 변경 미증명
- 현재 사용자 설치0.11.1 보존
