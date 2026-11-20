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
  - "repo:crates/hive-projection/src/lib.rs#sha256:2b119cea5c4b445e65dd63fb32a7a77ecb983b9579522dfdd0ec4dfa9e5ae7a3"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:0914190795ac1ac59555a994d0e39620060cdedd63b57230fe05b8c97b95cae5"
  - "repo:harness/skills/catalog.yml#sha256:7ceee94c26a31fd99960527c8570d9d35f97683aa0c97511493cc1494bef4078"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 37 Skills. Renamed IDs retain direct migration aliases;
`knowledge-scan` extracts new knowledge and `knowledge-transfer` moves existing knowledge.
`humanize-kor` adds explicit Korean rewriting with deterministic
preservation gates. Korean knowledge display names show only the human function, while every
description starts with its canonical ID. Historical release inventories remain immutable.
