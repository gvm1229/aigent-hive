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
  - "repo:.github/workflows/release-publish.yml#sha256:32d5b627460ec9f4881bb142e60559540a78fcbd7b7f461fc6f9f84808af3b05"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:85b13d22add18756fa11e29fcc1ebcf84b18d143385991143a8453c29e3d0328"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:1acc607d5b8703117b2f8b3e9e31c0f9a5f1653c4262a477441b6c078cf24d81"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:cf02204eafa02d03f95a147ae364548b1635e4c445192f3c0e67a38ed5104b8f"
  - "repo:harness/release/stable-skill-ledger.yml#sha256:8b2ca917aeb92cff8185221b07d93b450588ae668b7f506e844bc279d47f12b5"
  - "repo:scripts/check-stable-skill-ledger.py#sha256:b19da205df0303dc56e9e8ceeed1ac84f26db8abc337b7e75c0dc06e5a35ed24"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
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
