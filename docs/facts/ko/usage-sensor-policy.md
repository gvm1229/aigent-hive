---
schema_version: 1
pair_id: usage-sensor-policy
topic_slug: usage-sensor-policy
language: ko
counterpart: ../en/usage-sensor-policy.md
title: "Usage sensor 정책"
summary: "Qualified host-native sensor 우선과 optional CodexBar fallback."
tags: [sensor, usage]
aliases: ["Native-first usage"]
sources:
  - "repo:crates/hive-cli/src/main.rs#sha256:c7dfa1349f5239cd9df726f0fbf7d9a73a3da5a90218551cac420c0d7331a9bd"
  - "repo:crates/hive-cli/src/usage.rs#sha256:b8c12b63eb8011272e75592f2b5ac87a3c6aca78181fe223b6c462b14b332402"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:836d4ac233d8e719a9cd1b5a09a1deee7111a7c4e208f95f07e277fa3c899b98"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:0c6cf716cd863a1a15a1cb842be38302e1a8a803b068bcc2ea4e0e0b6d2316c1"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:e7afab494cd37c690cf68a4bd49d90a3251e4396"
status: active
---

# 사용량 조회 정책

- 검증된 호스트 조회 우선. CodexBar는 허용된 미지원·연결 불가에 동의 후 사용, 하한 판정 우회 금지
- 요청 계정 부재 시 완전한 검증 계정 하나에만 지문 없이 한 번 재조회. 누락·복수·손상·오래됨·하한은 차단 유지
- 신속 설정 하한20%, 일반 설정은 기본 조회 실패 후 보조 도구 동의 요청
-0.13.1 보조 조회: 새 기본 조회 프로세스, 경로별5초·같은 제공자
- Windows 고정 조회 코드:1,810초·115/115 정상·연결8개·보조 회복1회·허위60초0건. 새 앱 중단 수용 대기
