---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Aigent Hive has a 26-Skill product-only catalog; Korean knowledge Skill display names show only the human function, and descriptions start with the canonical English ID."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:2a4e830f797922b958d3f1ff934fd149c7242f550f6ae2841d664602231a427a"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:b5de8baa9c4973127ad34b6351c5478f4343c143e1d6cbaeed69a61638940a87"
  - "repo:harness/skills/catalog.yml#sha256:fc3facea5c95637482772e7a723fb98f17258b65eb8e6140c2cabe48afae7476"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:8fcf8b4794bb7d3d92065ad3f49a03acb33c4c13"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 26 Skills. Existing English IDs remain stable for
execution and setup compatibility. Korean knowledge display names show only the human function.
Each description starts with `(knowledge-...)`. `knowledge-capture` keeps one safe useful claim
after a turn; recall searches current work, import scans a chosen repository, promote shares
reviewed knowledge, and maintain checks or explicitly cleans it. This change is for a future
version. It does not change the `v0.9.4` release, tag, or package.
