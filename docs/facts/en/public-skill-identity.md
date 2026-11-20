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
  - "repo:crates/hive-projection/src/lib.rs#sha256:9561647510a75cdc259d93a93f032e09d620f41adab550291f1c93ef3b564ed4"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:92c6ae4cc0766a45e77a353a77592363b2e5b95ba8baa3f3c77338c144a17bb3"
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
