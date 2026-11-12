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
  - "repo:crates/hive-projection/src/lib.rs#sha256:e5300ab8f7f848f6fc6d6fb417d4823e70ea518663132ca05eec34232e5995b1"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:b1d168024659e23bc1fee30c46e2b628e607522b9b0da2f59229a277eff2a702"
  - "repo:harness/skills/catalog.yml#sha256:2598c30eb9870eba93ae14b5439c87c09d67c4a1b0cb2fabcdff8a40702871ba"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 37 Skills. Renamed IDs retain direct migration aliases;
`knowledge-scan` extracts new knowledge and `knowledge-transfer` moves existing knowledge.
`humanize-kor` adds explicit Korean rewriting with deterministic
preservation gates. Korean knowledge display names show only the human function, while every
description starts with its canonical ID. Historical release inventories remain immutable.
