---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Hive-owned source and consumer Skills share one short action-oriented naming policy, while consumer invocations retain the aigent-hive plugin namespace and retired IDs migrate fail closed."
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

# Skill Identity

Every Hive-owned source and consumer Skill participates in one reviewed naming set. Shared Skills
use the same short ID on both surfaces; source-only and consumer-only Skills stay on their proper
surface but follow the same action-oriented naming policy. Consumer host invocation remains
`aigent-hive:<name>`. Retired source and consumer IDs enter the transitive migration ledger, while
historical release bytes remain immutable and unverified old paths fail closed without writes.
