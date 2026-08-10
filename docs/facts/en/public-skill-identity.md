---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Aigent Hive uses one product-only 22-Skill catalog; source development reuses those Skills with repository directives, and retired source IDs migrate fail closed."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:d30564f33f2ead463cfe9e18aa68b697cb07b6c419ee42c9b583fcc11edaf966"
  - "repo:docs/plans/PLAN.md#sha256:ff2f68a6f812ba316c8999dce71e02576465d7d6956ad66a2181c52850d45a6b"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:5ae5f5e9e3ac9f2d9891393a75820fd1adbe8293e467a452d9778cba7fcb0468"
  - "repo:docs/skills.md#sha256:3ac35c43bee2bd83980415464b852253f271e95794d17fa81074fb2db0f88ec7"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:35f5bce71814a3e874fe53a8730024f16013ad46"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 22 Skills. Source development uses those installed
product Skills plus repository-owned directives; the final tracked source Skill count is zero.
Historical `hive-loop-engineering` maps to product `ralph-loop`. Product `ship` discovers each
repository's Git rules, while `amend-directive` changes user-owned behavior without weakening
compiled security and integrity boundaries.

Retired IDs support one-to-one, merge, split, or no-Skill routes. Source Wiki work routes through
the three product knowledge Skills and `hive source-wiki`; read-only code or Git inspection uses
the host's ordinary repository tools. Historical release bytes stay immutable, and unverified old
paths fail closed.
