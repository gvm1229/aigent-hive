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
  - "repo:crates/hive-projection/src/lib.rs#sha256:c19c4a8b62a64451974281a7651206b4c639d8dadd8d26c58bcad4669259f721"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:2a44b3975358e8df56e5e8d8e7f84ece2579aa073137cf9916f9e3cbee102cad"
  - "repo:harness/skills/catalog.yml#sha256:ffc547c4d245a37da39faa89f75900fc6dfb1ab8f4b9375d81289b087195b506"
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
