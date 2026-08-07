---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Public Skill Identity"
summary: "Consumer Skills use short action-oriented names under the aigent-hive plugin namespace, with selected-language descriptors and a fail-closed legacy migration."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/plans/PLAN.md#sha256:b584aa3e57a316c23de5df4a5f403a8daa36561220ac3199385f529b5a21ce0d"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:84d413a9632773e8a617cc40429ffecbb88a1c609cd9e96bd77ee431de594900"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:90624108d8774fea2ed71efe64a5263cbb14fbe5"
status: active
---

# Public Skill Identity

Consumer Skills use short action-oriented names and the host-provided `aigent-hive:<name>`
namespace. `record-knowledge` records one reviewed durable fact; `import-repository-knowledge`
performs a reviewed bulk repository onboarding. `clean-ai-slop` and `research-practices` complete
the consistent public naming set. Saved legacy IDs migrate to short names before validation; new
projections emit only current IDs. Hive-owned user projections render display names and concise
descriptions in the selected `en|ko` interface language. Historical release inventories retain
their exact old bytes; unauthenticated or overlapping local changes remain no-write conflicts.
