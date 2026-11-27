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
  - "repo:crates/hive-projection/src/lib.rs#sha256:231667ff59ebdf77e225a02c36e12a911a4aa4ce23eb5290d94f7f560306b662"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:c8a9253f37738a5031244de86ea5fc53ae2814ea9b76e1f4eef081741f26401e"
  - "repo:harness/skills/catalog.yml#sha256:6e817b733d042b45e8f1700d948c91fb58d75bb619e1c47ab381b9bd446ac652"
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
