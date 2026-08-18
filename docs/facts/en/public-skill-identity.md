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
  - "repo:crates/hive-projection/src/lib.rs#sha256:2fc1d83be7b6cfcf11ff0da628199b8d2c79642d2e69c7613778f4af81c847c2"
  - "repo:docs/plans/active/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:d5b65f1bed7b9d4adeaf168df3dc349de9c20b4b0fb84e09a14be95084012a71"
  - "repo:harness/skills/catalog.yml#sha256:d23ab5c0d658f432c1f051352ce9f21b4646e85f3bd45df0105d5559f386481c"
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
