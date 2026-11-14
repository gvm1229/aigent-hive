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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:55c9618f6cc2f1e5b18c00c2fc0c8260c6ada6c2020638a0ac2e81d6c0e4e4e0"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:ccdb3fbd321eef3dd9d91c412832203b68fd71bab30689c8a9095f6fd89520f3"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9903f35f13ae28c9853cdf99ab17c42876e8a5b12c91100e7f4c51cc04147c09"
  - "repo:crates/hive-projection/src/lib.rs#sha256:4d4d64b5b580a455b365a1039de568b39ff395072dc0d653e8ea495238a73fe4"
  - "repo:crates/hive-render/src/lib.rs#sha256:9859ba77fe6044c1da05931adedb5ffc1c9bb523b6953964438afb48a91c26fb"
  - "repo:docs/guides/installed-usage-guard.md#sha256:96e73fe5c33bb18cf1c2d6e21271df1738c02bc77f06a70fadbcb5b67ea8f5e5"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:627840988addc87245385867a8d11b921cea8ea7ed7ebd74257bb20f9515d928"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
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
