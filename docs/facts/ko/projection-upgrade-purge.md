---
schema_version: 1
pair_id: projection-upgrade-purge
topic_slug: projection-upgrade-purge
language: ko
counterpart: ../en/projection-upgrade-purge.md
title: "인증된 projection 갱신 정리"
summary: "Hive는 이전 Hive projection을 인증한 뒤 retired Skill과 직접 충돌하는 안전·소유권 규칙만 갱신"
tags: [consumer-harness, preservation, skills, upgrade]
aliases: ["PUG93"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:08d4aa0959ccc377a3f96a4c6f37df6f71c1473a271b406f7eb3b214f860cf0c"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:e1e23470bdd37528700da00641cedef9e510c525863c991c9d189ab761e9bde1"
  - "repo:crates/hive-update/src/merge.rs#sha256:a8eeefc6b27b42c7eb0c0795f4ca91b25401cbdfdd9f00064a629138a50e6283"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:786458e784a68b2ef7b7f1dc409ff36b4aea62b3419fe69551417175cee29429"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:d1105d7d3ebc2c5b8dbf100c4383ecd0933a8425c07501785b84ccdd4c6d2719"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:91f5fe71c2f45f7c6b5b47dcada9900f6561572e652689d6cffeed5973bc68c6"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
status: active
---

# 인증된 projection 갱신 정리

전역 설정은 retired-name ledger와 배포된 과거 Hive digest가 active byte와 모두 일치할 때만
`.agents/skills/<name>/SKILL.md`를 제거. 프로젝트 갱신은 인증된 project base inventory 사용.
incoming projection에 없는 미수정 retired 경로는 삭제, 수정·foreign byte는 보존.

Hive directive와 `AGENTS.md`의 Hive-owned marker는 safety·ownership 내용을 가진 incoming rule이
기존 Hive rule과 겹칠 때만 incoming 우선. 분리된 사용자 추가, foreign block, 안전과 무관한 겹침은
local 우선 유지. 모든 갱신에 preview·digest·atomic apply·rollback·빈 owned directory 정리 경계 적용.
