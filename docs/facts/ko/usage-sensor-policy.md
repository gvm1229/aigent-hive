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
  - "repo:crates/hive-cli/src/main.rs#sha256:0e64d62a5fd1a6d3586714d422107eb7def1a726c6cf4852bf3412c5ce56c42a"
  - "repo:crates/hive-cli/src/usage.rs#sha256:5ea98976584865f13743db52d8d8ce590d7f98af2693c14d48568dc4786ef753"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:71c3c2800d6d88c3366c125a948923d92066cb12f35db02c0594905976d7ac28"
  - "repo:crates/hive-cli/src/usage_control/manual.rs#sha256:56dfd0b134074e7797d3ab27b203800fce2809758460a40f0500e1ad7b39bd80"
  - "repo:crates/hive-cli/src/usage_monitor/sensor.rs#sha256:16030efe5ef70df9040e251fb46f05cd204e7819c42c692721cb5a5782f4cdeb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:04bab46c9c8e3aa4b7491d6ff2e3051d6fa5b4b4a16fc79e46808a6e3e619308"
  - "repo:crates/hive-core/src/preferences.rs#sha256:6ec5ee3711750ed82cd015e412ade49185152bec1831dbcbf077e52f122cce3b"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:docs/research/usage-guard-0.13.1.md#sha256:dfe5f7286915bdfb77cf392444bf70ae13fe2a1f54915ff915c73d5d1468937e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:tests/results/usage-current-app-0.13.1.json#sha256:107a03e8a27f3c3a64de9c34a9b74de7d83f8ced835b318a547d04914c9476c9"
  - "repo:tests/results/usage-sensor-soak-0.13.1.json#sha256:0765e3e50a78ec7b1d58be1fc030a0641ea82413ec1c43b46ddb335fe7079df7"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 사용량 조회 정책

- 검증된 기본 조회 우선. CodexBar는 허용된 미지원·연결 불가에 별도 동의, 하한 우회 금지
- 요청 계정 부재 시 완전한 인증 계정 하나에 지문 없이1회 재조회. 보호 활성 시 누락·복수·손상·만료·하한 차단
- 0.14.0 이전 빠른 설정20%·기본 조회 실패 후 보조 동의. 새 v2 보호 기본 비활성, 수동 조회는 관측만 수행
- 0.13.1 보조 조회: 새 기본 프로세스·경로별5초·같은 제공자. Windows 고정 조회1,810초·115/115 정상·연결8개·보조1회·허위60초0건, 추론·AI 하위 작업·도구·직접 재개 통과
