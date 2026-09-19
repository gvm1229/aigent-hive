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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:267309f9d7f56095b6cc00f4602aeaf1249e0e8988f7fc5bd897dff04e2be5f5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:crates/hive-projection/src/lib.rs#sha256:72d1a5158093bb93070183b12db1d2ce8388fd274f6b9a1fd21188ef7bac04b1"
  - "repo:crates/hive-render/src/lib.rs#sha256:d7ac376de1ffbfdf6f04f900fa8b46d78749755ffe9c30e1710be42611353ef7"
  - "repo:docs/guides/installed-usage-guard.md#sha256:9cf01b711909bca15472e95b5a25325b7094df4115795b6fdf04cf2bc3f017f5"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:0184c52dee665e90424d64cafa8c0a76e673d4a43019549e138879cc2085cafc"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:a1b96c88b4938c8771ccf635ffef0e8dcafa9ff1"
status: active
---

# `0.10.1` harness 갱신·사용량 정책 복구

- Project upgrade: historical full base 인증 → 상태 migration → current projection 생성
- Skill 선택: 선언된 merge만 통합, duplicate·unknown ID·tamper 거부
- `0.9.5`: `iterative-execution|ralph-loop` → `verified-workflow` 1개
- 공통 등록표: project·user exact digest, 생성형 공개 시험판 overlay chain, 동결 `v0.10.0` bytes
- Usage halt: effective policy digest 결합과 변경 threshold same-session 재검사
- 재검사: allow 때 old halt 제거, limited·unknown 때 교체, guard disable 없음
- 옛 동일 프로세스 복구 제한은 `0.10.3` 재측정으로 대체. `0.11.0` 초기화 확인은 별도 절차
