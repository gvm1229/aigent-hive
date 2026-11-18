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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:81c8223df0cba01511f0da69e8bc2b79bca26a22fa0439af874f6b4ed320606e"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:bebefffbf834ee4eefaeb371f1041e9a74a9eaba1aa51368461d5a425fedeb42"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d8306ac5295378b396357da94a7b6adc7579a63b3bcfa906d477bbeca8e2d4c0"
  - "repo:crates/hive-projection/src/lib.rs#sha256:82599d4eeb2477228ff3b76ef634a18be0a208c9ebaa2ec823b8c1da229d8a35"
  - "repo:crates/hive-render/src/lib.rs#sha256:d037c1cea2d7a4bef6b5476a0cac77a8b7113034ec135cc5043fc7e30416cf95"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b8bc06b557cc8582ecf988de4fce0c9c2aefd508d33adabcdb59668d078d988e"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:c56c61329f495702b7cad50d37fcc88921ad0fd381fc4486535b3b883f98995b"
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
