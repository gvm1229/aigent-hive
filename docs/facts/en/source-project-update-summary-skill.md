---
schema_version: 1
pair_id: source-project-update-summary-skill
topic_slug: source-project-update-summary-skill
language: en
counterpart: ../ko/source-project-update-summary-skill.md
title: "Source Project-Only Update Summary Skill"
summary: "The Aigent Hive source workspace has one nonshipping project-only Skill, update-summary, for verified Korean subscriber release summaries that exclude developer- and contributor-only changes."
tags: [development, release-notes, skill]
aliases: ["update-summary"]
sources:
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:8504ec054123dc8ea1b36383ab8ca3529c96cc4df6b8f7c948bfd21f09796a46"
  - "repo:docs/plans/active/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:26b949e1cfa5bfe4470693c7a1282100a9cb908e"
status: active
---

# Source Project-Only Update Summary Skill

`update-summary` is an explicit maintainer-authorized Skill for this source workspace. It writes
Korean subscriber updates from verified current and preceding stable-release evidence. It is not a
product Skill and does not enter `harness/`, a release bundle, a product catalog, or a consumer
projection. It includes only changes that alter a subscriber action, outcome, safety boundary, or
usable understanding. Release-description formatting, CI, verification records, repository plans,
and contributor workflows are excluded unless they directly change the installed product.
