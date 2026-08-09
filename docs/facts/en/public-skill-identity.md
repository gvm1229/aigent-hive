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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:5a60d254c760db58049da72530895a981708d549700b02656c7ff51224140f5f"
  - "repo:docs/plans/PLAN.md#sha256:2182e5c3942543533f9ae4b0b07d60449c83c46cbe379f23f6372c77afc7326e"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:7e8bb78ea5a37b0d4748de54a5e5816b9d4529c6f68844c8c1054859c47d3b4c"
  - "repo:docs/skills.md#sha256:45ee795d93d82e255355090e972f413d6c842076a51594c94b226837ec0bf125"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:6ed32f63fa3c67bed31164b9d15259f48443341a"
status: active
---

# Skill Identity

Hive-owned source and product Skills use distinct active IDs. Every approved source ID uses the
`source-*` prefix; product host invocation is `aigent-hive:<name>`. Historical
`hive-loop-engineering` maps through `engineer-run` to source `source-ralph-loop` and product
`ralph-loop`. Retired IDs use a scope-aware migration ledger; historical release bytes stay
immutable and unverified old paths fail closed.

Non-repository-specific source duplicates move to product counterparts. The single user-facing
guard is product `usage-guard`; source enforcement is an internal adapter.
