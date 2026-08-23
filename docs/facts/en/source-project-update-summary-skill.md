---
schema_version: 1
pair_id: source-project-update-summary-skill
topic_slug: source-project-update-summary-skill
language: en
counterpart: ../ko/source-project-update-summary-skill.md
title: "Source Project-Only Update Summary Skill"
summary: "The nonshipping source-project Skill update-summary creates verified Korean subscriber release summaries and stable-release Discord payloads while excluding developer-only changes."
tags: [development, release-notes, skill]
aliases: ["update-summary"]
sources:
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:457244a4c97b85e196053ecf36f42c033c24579dc7697d2fe64940f86f71aedf"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:f1c89f0998447f3bc53fbe0560521874efc65323"
status: active
---

# Source Project-Only Update Summary Skill

`update-summary` is an explicit maintainer-authorized Skill for this source workspace. It writes
Korean subscriber updates from verified current and preceding stable-release evidence. It is not a
product Skill and does not enter `harness/`, a release bundle, a product catalog, or a consumer
projection. It includes only changes that alter a subscriber action, outcome, safety boundary, or
usable understanding. Release-description formatting, CI, verification records, repository plans,
and contributor workflows are excluded unless they directly change the installed product.
For a stable release, the Skill saves its exact Korean payload in
`docs/releases/<version>.subscriber.ko.md`.
