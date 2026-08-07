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
  - "repo:README.md#sha256:30e7d1dece221c145e4a75fe9e05ec9520ca3ab58b7d1311088b9c4ad72759ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:128edc67999108258248cd5d1c356666931bbc7a6d9a747eaf108bc0cf5125f3"
  - "repo:harness/skills/setup-hive/SKILL.md#sha256:cb996a8698314710ce527c2c1d5bf41c0895bead8e7d52f9b1c4052b8d6666f6"
  - "repo:harness/user-setup/catalog.yml#sha256:af1147b8468f48eb81ec77ed4a14d5eba2fd31a4302e5459544fec3b2e22b595"
  - "repo:schemas/user-setup.schema.json#sha256:680009cadc1d41add4b16331bde37509cf636c845644a3923094a281110fb786"
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
