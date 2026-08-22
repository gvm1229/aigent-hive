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
  - "repo:README.md#sha256:362f1c802d9f436ffc33682d07709ed9655ce8fa098085f8d930fba93a84888e"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:786da31401085e9445495aa37defe7cedf781bc8457211a6addd23016c0bf922"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:1fcbb2b9b2db6d57bd40682f80db2a0a916ebbffb3434431038b609b6b743c11"
  - "repo:harness/user-setup/catalog.yml#sha256:167c45faf3724479bd83b48e6bc48074761c45ddf9160b7894742b291fbc503e"
  - "repo:schemas/user-setup.schema.json#sha256:d2985cbe53cc6aeb6a03442ca4af030e35dbfbda200c478b82b00f1c6b407cfa"
links: [global-onboarding, language-consistency]
reviewed_revision: "git:01df1d580d987e7fb0f34978076cd000263fd99f"
status: active
---

# Global User Contexts

Global setup stores any combination of web development, game development, and general knowledge
work, plus an optional short description. They describe the user, never a project workflow,
implementation choice, delivery priority, or Skill selection.

All built-in Skills are the default; users may instead toggle individual Skills. A saved legacy
single profile migrates to its matching context; a legacy custom profile retains its description.
Korean setup keeps `Skill`, `Wiki`, and host names as product terms: never translate `Skill` as `기술`.
