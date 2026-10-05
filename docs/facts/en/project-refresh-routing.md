---
schema_version: 1
pair_id: project-refresh-routing
topic_slug: project-refresh-routing
language: en
counterpart: ../ko/project-refresh-routing.md
title: "Project Refresh Skill Routing"
summary: "Selected global project-refresh Skills accept natural-language project guidance updates."
tags: [project, routing, skill]
aliases: ["project-refresh"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade/skill_merge.rs#sha256:9c3362827e588d15401dd0f11a4853ea02155749864615a9083ebdfdd8126ae3"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/project-refresh/agents/openai.yaml#sha256:b2563a605a8a14b629efb04dc36c7f4b4e4c556f91b5ea9c6cdb454bc92fccf8"
  - "repo:schemas/project-skill-merge.schema.json#sha256:d0e320be45bbc7873261a9b125359d87e883c54903b2a11ab13939c56404a599"
links: [project-onboarding, skill-routing]
reviewed_revision: "git:5857c29c72341a0eb8a9fc55e360259144432dee"
status: active
---

# Project Refresh Skill Routing

Selected global and project Skills support natural-language updates. Preview-only requests never apply. The Skill handles authenticated upgrade, exact path coordination, preservation and validation; users need not type CLI commands.

Reviewed merging combines incoming Hive improvements with user Skills. Fetch official inputs and review both body and companion settings. Apply requires the exact target/plan approval digest; changed local, incoming or merged content requires review again. Actual model selection needs separate host evidence.
