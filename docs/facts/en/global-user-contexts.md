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
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a4d69016bdf3c5f0e8ee75839c7076b804ce55a8583fa56aabd933545d148611"
  - "repo:harness/skills/setup-hive/SKILL.md#sha256:2851a369d75eaa79fe50ba9295787c09edbf7b25163e6fb64260ceba472db843"
  - "repo:harness/user-setup/catalog.yml#sha256:af1147b8468f48eb81ec77ed4a14d5eba2fd31a4302e5459544fec3b2e22b595"
  - "repo:schemas/user-setup.schema.json#sha256:b94594a2597f8eab3bcb778c24b892ee45c3856ce421043a79c3861b59cb99ee"
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
