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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2150681617bd1c2273780f0796609f27fc4815418428c0743ef11b88245deb38"
  - "repo:docs/plans/PLAN.md#sha256:e577012ddd2335521161f05ca14fef44a76b11c167ecb223b0cdf39aa8d30c9c"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:8f080cbf7902047256f1a8f789a7ba188817d4d86bbe5f57dfa9fa08484d31a8"
  - "repo:docs/skills.md#sha256:3ac35c43bee2bd83980415464b852253f271e95794d17fa81074fb2db0f88ec7"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
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
