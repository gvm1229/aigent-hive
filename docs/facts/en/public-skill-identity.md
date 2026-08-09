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
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:a6b7ee8e9da874952ae51002fb0fa79f0642612cbd311a3dc5f0de168540044e"
  - "repo:docs/skills.md#sha256:45ee795d93d82e255355090e972f413d6c842076a51594c94b226837ec0bf125"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:1e1cd88e59308bd5488275a3714f63c6c16cea52"
status: active
---

# Skill Identity

Every Hive-owned source and product Skill participates in one reviewed catalog, but active IDs are
distinct across the two surfaces so the installed product and the source workspace remain
unambiguous. Every approved source ID uses the `source-*` prefix; related workflows retain
recognizable name families. Product host invocation remains `aigent-hive:<name>`. Historical
`hive-loop-engineering` maps through current `engineer-run` to source `source-ralph-loop` and
product `ralph-loop`. Retired IDs enter a scope-aware transitive migration ledger; historical
release bytes remain immutable and unverified old paths fail closed.
