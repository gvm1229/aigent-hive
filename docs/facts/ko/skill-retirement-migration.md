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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:66a7fc4b9b18ba9fbe35e252b13188aa19c80488c76e4f573e7671f79aa07ead"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:354ea0ab7f22d94d231a5bb2c54385d003a04815"
status: active
---

# Skill 폐기 migration

- Direct jump: 중간 release 순차 설치 없음
- Stable registry: npm `0.8.0`, npm·GitHub `0.9.0–0.9.5` 영구 coverage와 future stable publication 전 snapshot append
- 구현 증거: stable 7개 release tag `active-skills.yml`의 digest·side-effect·capability registry
- 자동 제거: Exact authenticated Hive Skill·host projection·manifest entry·empty directory
- Local edit: Safe merge 가능
- Foreign·unknown bytes: 삭제 없이 conflict와 새 release activation 금지
- 성공 invariant: Discoverable retired Skill `0건`
