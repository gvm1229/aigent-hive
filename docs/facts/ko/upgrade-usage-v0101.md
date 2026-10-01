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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:37b93ac3de38edc4d3476786c58a3be0dec34824c39b8eb7d3fc9b7eb39f8c94"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:42f69c326667ad73522caeadec761a523074a78efcb5caf05a74185acb0fe3ce"
  - "repo:crates/hive-projection/src/lib.rs#sha256:367b2792e83334e0c4b4d1a3ddbe1c20293dec5427ed8713320c580c62a6b313"
  - "repo:crates/hive-render/src/lib.rs#sha256:9c9cc1123e0d36f14863eaeaa7e61eb5dd9b1e37b033d672dc821bef7b3c7157"
  - "repo:docs/guides/installed-usage-guard.md#sha256:c94975f1e11052ebf9c04e00066fe121229eea186945c056164d3a33c609df87"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
  - "repo:harness/project-bases/registry.yml#sha256:bdf87d414f2fbfca1d69132601f1c1f0a1a488798beebe72c06db772fb10d019"
links: [historical-project-base-coverage, installed-usage-guard, skill-retirement-migration, usage-guard-thresholds]
reviewed_revision: "git:bb177081c93b9d196d9dc3e07acefae9908c7d0e"
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
