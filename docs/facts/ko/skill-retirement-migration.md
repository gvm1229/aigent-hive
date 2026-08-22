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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:fe327177fca73ccbdb3267a1cfca7b579b984e8bd3a24e74457a7d062020f2ec"
  - "repo:docs/decisions/product-release-decisions.md#sha256:59e330c3bd0a5a8133e00c447c99db44e30274dbf92770b662d3cf4c14b50e0f"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:48117baa1a1de7d252946c6f270c6eef67b526223e954dab98fb5c585bd3b8a2"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:26e5fd299f961d79c6b8237c212b4b07e9e99770"
status: active
---

# Skill 폐기 migration

- Direct jump: 중간 release 순차 설치 없음
- 자동 제거: Exact authenticated Hive Skill·host projection·manifest entry·empty directory
- Local edit: Safe merge 가능
- Foreign·unknown bytes: 삭제 없이 conflict와 새 release activation 금지
- 성공 invariant: Discoverable retired Skill `0건`
