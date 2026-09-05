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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:a9535c1ae9e207b08dfce0d71fe9293e38168d8c7a38a4187ce8c6752be90ce4"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:66dc337fae3ced831c3775915aef1b83c6406314cee873f20cf81e75d3c826cf"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:5a7d279c9ce96bec6792f191e4be0caa69bf17c9062ad535e11478af7f4408c2"
  - "repo:crates/hive-projection/src/lib.rs#sha256:274d1221abb312197451cd8afc55a45eda881d08980b932d87454659b46c562d"
  - "repo:crates/hive-render/src/lib.rs#sha256:4ce1a5feac500ede6f71c6b1b2ba0764e189ab48b6cae971122e8d5e538eee42"
  - "repo:harness/project-bases/registry.yml#sha256:3e53d7ee8a72c0dfff784927fa8a22ecbfef6f19dd0f77d76d3772c5043d6779"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:ecd92340604353fd0935e8f0c4ccdcd2b34288f7"
status: active
---

# `0.10.1` harness 갱신·사용량 정책 복구

- Project upgrade: historical full base 인증 → 상태 migration → current projection 생성
- Skill 선택: 선언된 merge만 통합, duplicate·unknown ID·tamper 거부
- `0.9.5`: `iterative-execution|ralph-loop` → `verified-workflow` 1개
- 공통 등록표: project·user exact digest와 동결 `v0.10.0` bytes
- Usage halt: effective policy digest 결합과 변경 threshold same-session 재검사
- 재검사: allow 때 old halt 제거, limited·unknown 때 교체, guard disable 없음
