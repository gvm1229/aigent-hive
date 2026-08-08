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
  - "repo:docs/plans/PLAN.md#sha256:aaaf40a2618a888822eeaf404621966e1e79237916ff487621f90cd69e6da480"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:7a5ce0b43d26fe195dc2dd1f0bd2f4a8d578ae2115caa46f88856eb279e5c0e7"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:90624108d8774fea2ed71efe64a5263cbb14fbe5"
status: active
---

# Public Skill Identity

Consumer Skills use short action-oriented names in the `aigent-hive:<name>` namespace. Saved
retired IDs migrate to current IDs; new projections emit current IDs only. The canonical retired-ID
ledger resolves and reserves names, but a frozen historical release inventory or installed ownership
manifest is the only deletion authority. Changed, unknown, or foreign old paths fail closed without
writes. Hive-owned display names and descriptions use the selected `en|ko` interface language.
