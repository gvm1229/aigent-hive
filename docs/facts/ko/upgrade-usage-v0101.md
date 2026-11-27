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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:b9439680a397b8459548cffabf6ebd0ff74f5e5ca4977ec8adda5e9676f92a0b"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:50bbe234dff657b08dca42bc8cb17330a75d6aee436b5debd318fb039fd787c4"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:cd8fc9b3187b4d3395847b367ef455ba6be3b27a7140c720552f76f72eff7f3e"
  - "repo:crates/hive-projection/src/lib.rs#sha256:f315ebfc7c8d5e2902621d707f214a8b3a6564d198de1391ba9b82afd2e9e8d1"
  - "repo:crates/hive-render/src/lib.rs#sha256:e509d5d68ae22f608fe10a3bcd14efadf48c83cce6483e5cb87ef0ff6cef1b96"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b8bc06b557cc8582ecf988de4fce0c9c2aefd508d33adabcdb59668d078d988e"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:6e9425b04b25538acca9e51671d3218789a5d42a44c89a79d9e97c69f06246d3"
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
