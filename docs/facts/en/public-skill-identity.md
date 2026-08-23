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
  - "repo:crates/hive-projection/src/lib.rs#sha256:72c05f8bc6c5a7c3f94a42c34d87828c3beea0d32345171dac598d83a153819b"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:9d445726c92856de8c47781743cfc972aeeebbd1e74cc1660b860dd0ebac573a"
  - "repo:harness/skills/catalog.yml#sha256:d07890ccf090177ed03405d9eae01c278130cdd1fa9797ad3616106d1c67f6c8"
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
