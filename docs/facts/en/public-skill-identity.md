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
  - "repo:crates/hive-projection/src/lib.rs#sha256:9248e3cb42168040fa5cc9ce033eeb4e5446e216c7a902f12162bc7d48fd2691"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:c0faff5fbecfd6cde693ccec60ea07412eac4502c4cb2d5d6e3ced1bf83ee746"
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
