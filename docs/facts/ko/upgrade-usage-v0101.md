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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:66dc337fae3ced831c3775915aef1b83c6406314cee873f20cf81e75d3c826cf"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6d2138ea9d68803f4447e7295cc08fcf5b548c23df8b2d1e5826c0aa46bff668"
  - "repo:crates/hive-projection/src/lib.rs#sha256:69f96abcee52980e03fe542e335a02181e2f03b42218bdaa12b8acc6213dee51"
  - "repo:crates/hive-render/src/lib.rs#sha256:4e68aec9b3386fcf30cc49629a69614ef08f64cbc7b974db89cc279e52c60cc1"
  - "repo:harness/project-bases/registry.yml#sha256:f39a79033c230819250a0069dab4ad4eab29e3caac5607d7de6b0d5c238cd4a4"
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
