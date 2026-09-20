---
schema_version: 1
pair_id: skill-retirement-migration
topic_slug: skill-retirement-migration
language: ko
counterpart: ../en/skill-retirement-migration.md
title: "Skill 폐기 migration"
summary: "모든 지원 predecessor의 authenticated retired Skill artifact를 direct 0.10.0 upgrade에서 제거하고 foreign bytes는 activation conflict로 보존하는 계약"
tags: [migration, skills, upgrade, v0-10]
aliases: ["Retired Skill cleanup"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:e664105a2734fc5ec7c35f93ddc5ce0362ad5e391ae881c63e326a8c25866bca"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/decisions/product-release-decisions.md#sha256:9f234ef3fede8030ab6ad57fa4b560a71f30f468622c4e4ad56b83864d0e05ce"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:3e2106b90defce8839164efed8054463a8504b873abcb7cd07d7e8a8a45c60bc"
  - "repo:harness/release/stable-skill-ledger.yml#sha256:1bbe29fda34d23fc2a3accb4bc30714f68b2d1c1f1a7522850d09fe03843de73"
  - "repo:scripts/check-stable-skill-ledger.py#sha256:0aa6a3582d31854106dd69d7faf05072a9027a3e3cfa707e00fce43b7090de58"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
status: active
---

# Skill 폐기 migration

- Direct jump: 중간 release 순차 설치 없음
- Stable registry: npm `0.8.0`, npm·GitHub `0.9.0–0.9.5` 영구 coverage와 future stable publication 전 snapshot append
- 구현 증거: Stable tag plugin 기준본·digest registry와 18개 `0.9.x` direct-jump 조합
- 게시 gate: npm·GitHub 공개 stable 합집합·target·compatibility epoch exact parity
- lifecycle ledger: `0.10.0` rename·merge의 version·직접 replacement·collision fail-closed
- 자동 제거: Exact authenticated Hive Skill·host projection·manifest entry·empty directory
- Local edit: Safe merge 가능
- Foreign·unknown bytes: 삭제 없이 conflict와 새 release activation 금지
- 성공 invariant: Discoverable retired Skill `0건`
