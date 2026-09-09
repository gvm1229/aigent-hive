---
schema_version: 1
pair_id: upgrade-usage-v0101
topic_slug: upgrade-usage-v0101
language: ko
counterpart: ../en/upgrade-usage-v0101.md
title: "0.10.1 harness 갱신·사용량 정책 복구"
summary: "Historical project state 선인증 migration과 guard disable 없는 변경 threshold 재검사"
tags: [migration, project-upgrade, usage, v0-10-1]
aliases: ["0.10.1 upgrade repair"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:d5e36f5d1cb6080fa7952b1cf4354e7d54f0df12bc0799d758ced53d7f083b84"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:881ae77507817b888cf66ff0da2ee52fbc4048fbb9355641a093dcf6f3d69fc1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6d2138ea9d68803f4447e7295cc08fcf5b548c23df8b2d1e5826c0aa46bff668"
  - "repo:crates/hive-projection/src/lib.rs#sha256:c80df9705880a64e92a2af923ac394fccbdbd19385b4edd8fecfbf7f8c0dce67"
  - "repo:crates/hive-render/src/lib.rs#sha256:b32ec4b80e8196069441c3f6658a6167163b33439ea1859b0349f76b56ccc823"
  - "repo:harness/project-bases/registry.yml#sha256:195103d23c1539bcdb6cd3a84807e52306ed772f8a2fcbbf3f97ab3a56ece039"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:fede7a2a753884a414766773c6cf197721937028"
status: active
---

# `0.10.1` harness 갱신·사용량 정책 복구

- Project upgrade: historical full base 인증 → 상태 migration → current projection 생성
- Skill 선택: 선언된 merge만 통합, duplicate·unknown ID·tamper 거부
- `0.9.5`: `iterative-execution|ralph-loop` → `verified-workflow` 1개
- 공통 등록표: project·user exact digest, 생성형 공개 시험판 overlay chain, 동결 `v0.10.0` bytes
- Usage halt: effective policy digest 결합과 변경 threshold same-session 재검사
- 재검사: allow 때 old halt 제거, limited·unknown 때 교체, guard disable 없음
- 복구 범위: 같은 프로세스의 정책 변경; 다른 프로세스 표식은 안전 오류, 같은 정책 중단은 재사용
