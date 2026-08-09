---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Skill identity"
summary: "Hive 소유 source·consumer Skill 전체의 짧은 동작 이름, consumer plugin namespace, 폐기 ID의 안전한 이관 계약."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:d6cc73ae1bd278e0e9b2e06468cfcade31c0f731ef543a8ca84a5356b4aaa905"
  - "repo:docs/plans/PLAN.md#sha256:c8b0d218a2bad93e549048227a966c2dd94e88e365e00cf04abdc91f69274dbd"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:d049c26342e6b5ed595b33066eda61f6cc44b1f8f6494366e843c3b952d134c5"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:8bc8329e7a89a51848f4cea135bdc617b0164df2"
status: active
---

# Skill identity

Hive 소유 source·consumer Skill 전체: 하나의 검토된 이름 세트 적용. Shared Skill: 양쪽에서 동일한
짧은 ID. Source-only·consumer-only Skill: 각 위치 유지와 같은 동작 이름 원칙 적용. Consumer host
호출: `aigent-hive:<name>` 유지. 폐기 source·consumer ID: transitive migration ledger 편입.
Historical release byte: 변경 금지. 검증 불가 old path: write 없는 conflict.
