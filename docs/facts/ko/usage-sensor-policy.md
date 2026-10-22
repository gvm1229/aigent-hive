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
  - "repo:crates/hive-cli/src/main.rs#sha256:53492315e69a8d63cfa180d672e2557eb99b9102a7606998462eb334f192dd8f"
  - "repo:crates/hive-cli/src/usage.rs#sha256:b8c12b63eb8011272e75592f2b5ac87a3c6aca78181fe223b6c462b14b332402"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6a8b7909e224bde9bcf45ea91e8267219b6a7b1a4546f3b601705db94d144a88"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a7caad0d660d83378b297ec3678312fa741b411b8115ed9eca98400d1db703d0"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:tests/results/usage-current-app-0.13.1.json#sha256:107a03e8a27f3c3a64de9c34a9b74de7d83f8ced835b318a547d04914c9476c9"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# 사용량 조회 정책

- 검증된 호스트 조회 우선. CodexBar는 허용된 미지원·연결 불가에 동의 후 사용, 하한 판정 우회 금지
- 요청 계정 부재 시 완전한 검증 계정 하나에만 지문 없이 한 번 재조회. 누락·복수·손상·오래됨·하한은 차단 유지
- 신속 설정 하한20%, 일반 설정은 기본 조회 실패 후 보조 도구 동의 요청
-0.13.1 보조 조회: 새 기본 조회 프로세스, 경로별5초·같은 제공자
- Windows 고정 조회 코드:1,810초·115/115 정상·연결8개·보조 회복1회·허위60초0건. 현재 Windows 추론·AI 하위 작업·도구·직접 재개 수용 통과
