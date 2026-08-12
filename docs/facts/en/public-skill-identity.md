---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Aigent Hive uses one product-only 22-Skill catalog, and its Codex plugin identifies Hojin (Tom) Jeong and the official Hive mark."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/plans/PLAN.md#sha256:8f5e2be89e2655edce32efe025b0338761f34676221a68aedefd89d35f583b4a"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:5ae5f5e9e3ac9f2d9891393a75820fd1adbe8293e467a452d9778cba7fcb0468"
  - "repo:docs/skills.md#sha256:e472e29b807d09b86cf291821b1f75532c8c05d7f9359ba7bb2f6ebfb1fb7a93"
  - "repo:harness/plugins/aigent-hive/.codex-plugin/plugin.json#sha256:5701b2380c96d72245e21ef04aad028bb24b3f2d58c93d1581463ea21d230b6a"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:daa4ab56b05f7403bc1f5f2b44d8471fb99866af"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 22 Skills. Source development combines the installed
Skills with repository directives; tracked source Skills remain zero. Retired IDs migrate through
reviewed routes, historical release bytes stay immutable, and unverified paths fail closed.

The Codex plugin identifies `Hojin (Tom) Jeong` as author and developer. Its `logo` and
`composerIcon` use a centered 512 px crop of the official colored Hive mark and Hive gold. Request
context: replace the anonymous attribution and compass icon in `0.9.1`.
