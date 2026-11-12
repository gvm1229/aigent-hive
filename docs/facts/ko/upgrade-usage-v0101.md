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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:1ab23dbb5a8b60322f6c1068ba7cc09f54bc34151915068a748a1cc5a2c38018"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:9b858bef8831bed659b0f977ecd9ba84111800176d9dfc048b1d46c4c6ab8115"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d47f8ba8f5e3e88783cfc992cb88475d7cf47fa3459baecb434610e7f91c2d45"
  - "repo:crates/hive-projection/src/lib.rs#sha256:e5300ab8f7f848f6fc6d6fb417d4823e70ea518663132ca05eec34232e5995b1"
  - "repo:crates/hive-render/src/lib.rs#sha256:b6285195847e5b1779bd4a168315a5337eac5885ebacabce45171b22959bf199"
  - "repo:docs/guides/installed-usage-guard.md#sha256:0ead68f33595fcbd18d4d9c9ec38926645ca3cf22ffd895a02628c5e3c97a384"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:20b4d2b1f406e448c417841f5d71d033fb8705877314beb33ca6db241a3aed35"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
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
