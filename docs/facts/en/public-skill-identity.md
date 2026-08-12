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
  - "repo:docs/plans/PLAN.md#sha256:5b78a2a5c9e03e7f137f22dec46932e4739e67c0c793125ff19859b18e0c7cfa"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:5ae5f5e9e3ac9f2d9891393a75820fd1adbe8293e467a452d9778cba7fcb0468"
  - "repo:docs/skills.md#sha256:3ac35c43bee2bd83980415464b852253f271e95794d17fa81074fb2db0f88ec7"
  - "repo:harness/plugins/aigent-hive/.codex-plugin/plugin.json#sha256:7dd2d6cadb8f19f0dd2055fab2d56b93e6078b923a4fc7295578b609c994e696"
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
