---
schema_version: 1
pair_id: source-project-update-summary-skill
topic_slug: source-project-update-summary-skill
language: en
counterpart: ../ko/source-project-update-summary-skill.md
title: "Source Project-Only Update Summary Skill"
summary: "The Aigent Hive source workspace has one nonshipping project-only Skill, update-summary, for verified Korean subscriber release summaries."
tags: [development, release-notes, skill]
aliases: ["update-summary"]
sources:
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:2823028c4400acd719437475a8b122c8a68271d48f4a8a3f70410f8c21af2616"
  - "repo:docs/plans/active/source-update-summary-skill.md#sha256:29f2c05644f9a7fe7f418f5516fa70421708352fba3f66cf77564932de1a196d"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:26b949e1cfa5bfe4470693c7a1282100a9cb908e"
status: active
---

# Source Project-Only Update Summary Skill

`update-summary` is an explicit maintainer-authorized Skill for this source workspace. It writes
Korean subscriber updates from verified current and preceding stable-release evidence. It is not a
product Skill and does not enter `harness/`, a release bundle, a product catalog, or a consumer
projection.
