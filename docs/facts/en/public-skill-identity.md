---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Aigent Hive has 28 product Skills, direct rename aliases, and separate knowledge scanning and transfer roles."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:f315ebfc7c8d5e2902621d707f214a8b3a6564d198de1391ba9b82afd2e9e8d1"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:d018250efa539142d56f516c0f73202321ec4d38b6737260a348d513cd9fa266"
  - "repo:harness/skills/catalog.yml#sha256:e8eb787c8696d6ed34cc958110807294cd75d20431dcbd37083cb0d848885f78"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 31 Skills. Renamed IDs retain direct migration aliases;
`knowledge-capture` records and extracts new knowledge and `knowledge-manage` maintains, shares and moves it.
`humanize-kor` adds explicit Korean rewriting with deterministic
preservation gates. Korean knowledge display names show only the human function, while every
description starts with its canonical ID. Historical release inventories remain immutable.
