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
  - "repo:crates/hive-projection/src/lib.rs#sha256:2ebda9014906433468b9b07a89627938f32aedfe08f56e0203539d8b9e3a3ad9"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:c8436b3c92a4b0e3b11ea57d4cb1ea9b9f410b518a83feff81a31685d1cc0559"
  - "repo:harness/skills/catalog.yml#sha256:c0b54d4bd885f7f40cca464be778c30d5c594d7ffa94b9c09952297b8d5b6f51"
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
