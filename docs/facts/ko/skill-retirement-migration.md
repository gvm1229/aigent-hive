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
  - "repo:.github/workflows/release-publish.yml#sha256:241b1121f2b95690858514f9bcc8af27f6bd9084d125b14ff0e5df31cd401a3d"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:59e8e730d66ad5ace25e65b3476efeb7dc6b15ac1ba0b27d3666b98beece385c"
  - "repo:docs/archive/plans/skill-retirement-migration-0.10.0.md#sha256:3e2106b90defce8839164efed8054463a8504b873abcb7cd07d7e8a8a45c60bc"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:71a8c4e0f75fdc3f99aa7f4d1c4e2ace759af640dbc4dd013174639e71a3f01b"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:harness/release/stable-skill-ledger.yml#sha256:ae6f347eee9ea932aae78a9534b0844118d84a537682ae55012062d396cd1eab"
  - "repo:scripts/check-stable-skill-ledger.py#sha256:0aa6a3582d31854106dd69d7faf05072a9027a3e3cfa707e00fce43b7090de58"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
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
