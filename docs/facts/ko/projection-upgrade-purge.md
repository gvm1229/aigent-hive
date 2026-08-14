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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:8943d5559309ea5b084f211a4bda523bc88e1e5f6afdd23b6b1226e85a652bf5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9fa9e439ad15ea6a8b5ed7cf6d031595a8979b056dada55360cb32331d9e8355"
  - "repo:crates/hive-update/src/merge.rs#sha256:4dc96d4c159d55be6664fa565dbb0eb77c1df532330f8a539f028ce51a9fcaaa"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:3810a0ce4919ccbcfd02961a1cefdd5f6329d938eed1b411d839edeac3b3a86b"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:640be28ec7f75444a52544b0d36c45363696dcbd0281f9c5aabd0768d185784e"
  - "repo:tests/conformance/test_phase3_static_contracts.py#sha256:71224482ebc778130104504faf981d59b25e5c9790e7dec89f358b347fcd2e53"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:65f5a7df6d1abed4f9e299992d85e6377464b1d5"
status: active
---

# 인증된 projection 갱신 정리

전역 설정은 retired-name ledger와 배포된 과거 Hive digest가 active byte와 모두 일치할 때만
`.agents/skills/<name>/SKILL.md`를 제거. 프로젝트 갱신은 인증된 project base inventory 사용.
incoming projection에 없는 미수정 retired 경로는 삭제, 수정·foreign byte는 보존.

Hive directive와 `AGENTS.md`의 Hive-owned marker는 safety·ownership 내용을 가진 incoming rule이
기존 Hive rule과 겹칠 때만 incoming 우선. 분리된 사용자 추가, foreign block, 안전과 무관한 겹침은
local 우선 유지. 모든 갱신에 preview·digest·atomic apply·rollback·빈 owned directory 정리 경계 적용.
