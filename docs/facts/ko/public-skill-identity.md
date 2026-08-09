---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Skill identity"
summary: "Hive 소유 source·product Skill의 관련 있지만 서로 다른 active ID, product plugin namespace, 폐기 ID의 안전한 이관 계약."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:5a60d254c760db58049da72530895a981708d549700b02656c7ff51224140f5f"
  - "repo:docs/plans/PLAN.md#sha256:2182e5c3942543533f9ae4b0b07d60449c83c46cbe379f23f6372c77afc7326e"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:7e8bb78ea5a37b0d4748de54a5e5816b9d4529c6f68844c8c1054859c47d3b4c"
  - "repo:docs/skills.md#sha256:45ee795d93d82e255355090e972f413d6c842076a51594c94b226837ec0bf125"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:6ed32f63fa3c67bed31164b9d15259f48443341a"
status: active
---

# Skill identity

Hive 소유 source·product Skill: active ID 분리. 승인 source ID: `source-*` 접두사 필수. Product host
호출: `aigent-hive:<name>`. Historical `hive-loop-engineering`: `engineer-run`을 거쳐 source
`source-ralph-loop`·product `ralph-loop`로 이관. 폐기 ID: scope-aware migration ledger 편입.
Historical release byte: 변경 금지. 검증 불가 old path: write 없는 conflict.

비저장소 공통 source workflow: product 대응 Skill로 통합. 사용자 사용량 보호: product
`usage-guard` 하나. Source guard: internal enforcement adapter.
