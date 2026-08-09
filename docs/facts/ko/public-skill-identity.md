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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:fa37bbe1c62b968b8d76d56e6f094f48317f6d6cd3262056171f0518e9f468ea"
  - "repo:docs/plans/PLAN.md#sha256:71445da99eafe5fbbd3186fef770509dbc59ba62cb0ffbfe39eb32de3d545d52"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:4ffd0b48ab378c828051d8630687aca4a2d52d66a32959a5865a9f25e5043489"
  - "repo:docs/skills.md#sha256:b5d62d9a6fa6d1eba735862b1a930018ddc4f7cc169a698705ff6e8a0969234c"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:536f5076534cedcdb9ea3d118830792fe61cd75e"
status: active
---

# Skill identity

Hive 소유 source·product Skill 전체: 하나의 검토된 목록 적용. 두 영역의 active ID는 설치 제품과
source workspace를 혼동하지 않도록 서로 다르게 유지. 관련 workflow: 알아볼 수 있는 이름 계열 유지.
Product host 호출: `aigent-hive:<name>`. Historical `hive-loop-engineering`: current
`engineer-run`을 거쳐 source `source-ralph-loop`·product `ralph-loop`로 이관. 폐기 ID: scope-aware
transitive migration ledger 편입. Historical release byte: 변경 금지. 검증 불가 old path: write 없는
conflict.
