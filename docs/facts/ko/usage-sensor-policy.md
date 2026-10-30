---
schema_version: 1
pair_id: usage-sensor-policy
topic_slug: usage-sensor-policy
language: ko
counterpart: ../en/usage-sensor-policy.md
title: "사용량 조회 정책"
summary: "검증된 기본 조회·동의한 보조 조회와 v2 비활성 보호·수동 관측의 구분"
tags: [sensor, usage]
aliases: ["Native-first usage"]
sources:
  - "repo:crates/hive-cli/src/main.rs#sha256:dc14d1ddb9b68d254c1b3548c2011e29d07efb4ed7b4259fd708dd9aa95317a8"
  - "repo:crates/hive-cli/src/usage.rs#sha256:ce45a82f02376c71b9bd25a7daf377c11fa1c9b55a4f08098098f97cd7ae920f"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:9b858bef8831bed659b0f977ecd9ba84111800176d9dfc048b1d46c4c6ab8115"
  - "repo:crates/hive-cli/src/usage_control/manual.rs#sha256:86b216d3850bd88b428d1d9ea6407c11c6287f5c3e401926882e8d9e9d48d54d"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3c6e0b3a0f62d5598e68024fd7a69d82cb2b0dcfbb29d5602f6f362bc0df2d40"
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
  - "repo:tests/results/usage-current-app-0.13.1.json#sha256:107a03e8a27f3c3a64de9c34a9b74de7d83f8ced835b318a547d04914c9476c9"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 사용량 조회 정책

- 검증된 기본 조회 우선. CodexBar는 허용된 미지원·연결 불가에 별도 동의, 하한 우회 금지
- 요청 계정 부재 시 완전한 인증 계정 하나에 지문 없이1회 재조회. 보호 활성 시 누락·복수·손상·만료·하한 차단
- 0.14.0 이전 빠른 설정20%·기본 조회 실패 후 보조 동의. 새 v2 보호 기본 비활성, 수동 조회는 관측만 수행
- 0.13.1 보조 조회: 새 기본 프로세스·경로별5초·같은 제공자. Windows 고정 조회1,810초·115/115 정상·연결8개·보조1회·허위60초0건, 추론·AI 하위 작업·도구·직접 재개 통과
