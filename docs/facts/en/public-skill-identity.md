---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Hive-owned source and product Skills use related but distinct active IDs, while product invocations retain the aigent-hive plugin namespace and retired IDs migrate fail closed."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:fa37bbe1c62b968b8d76d56e6f094f48317f6d6cd3262056171f0518e9f468ea"
  - "repo:docs/plans/PLAN.md#sha256:71445da99eafe5fbbd3186fef770509dbc59ba62cb0ffbfe39eb32de3d545d52"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:4ffd0b48ab378c828051d8630687aca4a2d52d66a32959a5865a9f25e5043489"
  - "repo:docs/skills.md#sha256:b5d62d9a6fa6d1eba735862b1a930018ddc4f7cc169a698705ff6e8a0969234c"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:536f5076534cedcdb9ea3d118830792fe61cd75e"
status: active
---

# Skill Identity

Every Hive-owned source and product Skill participates in one reviewed catalog, but active IDs are
distinct across the two surfaces so the installed product and the source workspace remain
unambiguous. Related workflows retain recognizable name families. Product host invocation remains
`aigent-hive:<name>`. Historical `hive-loop-engineering` maps through current `engineer-run` to
source `source-ralph-loop` and product `ralph-loop`. Retired IDs enter a scope-aware transitive
migration ledger; historical release bytes remain immutable and unverified old paths fail closed.
