---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Public Skill identity"
summary: "Consumer Skill의 짧은 동작 이름, aigent-hive plugin namespace, 선택 언어 descriptor, fail-closed legacy migration 계약."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/plans/PLAN.md#sha256:b584aa3e57a316c23de5df4a5f403a8daa36561220ac3199385f529b5a21ce0d"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:84d413a9632773e8a617cc40429ffecbb88a1c609cd9e96bd77ee431de594900"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:90624108d8774fea2ed71efe64a5263cbb14fbe5"
status: active
---

# Public Skill identity

Consumer Skill: 짧은 동작 이름과 host 제공 `aigent-hive:<name>` namespace. `record-knowledge`:
검토된 durable fact 1개 기록. `import-repository-knowledge`: 검토 기반 repository bulk onboarding.
`clean-ai-slop`·`research-practices`: 일관된 public name 완성. 저장된 legacy ID: validation 전 short
name 이관, 새 projection 출력: current ID만 사용. Hive-owned user projection의 display name·short
description·`SKILL.md` frontmatter description: 선택 `en|ko` interface language 적용. Historical release
inventory: exact old byte 보존. Unauthenticated·overlap local change: no-write conflict 유지.
