---
schema_version: 1
pair_id: global-user-contexts
topic_slug: global-user-contexts
language: en
counterpart: ../ko/global-user-contexts.md
title: "Global User Contexts"
summary: "Global setup stores multiple user contexts as background only and preserves Korean product terms."
tags: [bootstrap, communication, onboarding]
aliases: ["User contexts", "User profile"]
sources:
  - "repo:README.md#sha256:413ed120770591773c5efab11aa1bc3587687b411eff47a665802b5bf0f5ea2b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6de1cf5f473fc0c6e61504b07ac8eb892abb77231b406d7952dc271e0ee23c1b"
  - "repo:harness/skills/configure/SKILL.md#sha256:17a80a35d5f367421c661374dec54147d0cabb4f48c4c5a640b15253bd5f0222"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:87bb452a4240faccdef5c96488b7492c3764f44a2819e8e7733b8c41dadc70b9"
links: [global-onboarding, language-consistency]
reviewed_revision: "git:a679bb4d1ea439ef172e8a7f59b649d6d34a1983"
status: active
---

# Global User Contexts

Global setup stores any combination of web development, game development, and general knowledge
work, plus an optional short description. They describe the user, never a project workflow,
implementation choice, delivery priority, or Skill selection.

All built-in Skills are the default; users may instead toggle individual Skills. A saved legacy
single profile migrates to its matching context; a legacy custom profile retains its description.
Korean setup keeps `Skill`, `Wiki`, and host names as product terms: never translate `Skill` as `기술`.
