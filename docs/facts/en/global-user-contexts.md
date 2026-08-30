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
  - "repo:README.md#sha256:27679c3c338ef2f82b352800ccb882c2536bcc2c7dbfd18b93df52e3349554b0"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ab2aaa4dd8d3ec7e90c366a65cf131b6eb2401f1b0b2c95c87d4a6448c7b3bd9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:a6aea1ed5b977bc818bace5c9d712d2da01328f59753e9b93136c17b1a8f24d3"
  - "repo:harness/user-setup/catalog.yml#sha256:eaeebd5ebb3dc7ea7bd1be287d916991ddbf8820264e744e331af44be3903ec2"
  - "repo:schemas/user-setup.schema.json#sha256:06ed2a954a7c98817a1288a29f779c1db45cfafa2ea21d8227695a1d988b5fb6"
links: [global-onboarding, language-consistency]
reviewed_revision: "git:1b755a995d91739d758830210d93cdc012e9e61b"
status: active
---

# Global User Contexts

Global setup stores any combination of web development, game development, and general knowledge
work, plus an optional short description. They describe the user, never a project workflow,
implementation choice, delivery priority, or Skill selection.

All built-in Skills are the default; users may instead toggle individual Skills. A saved legacy
single profile migrates to its matching context; a legacy custom profile retains its description.
Korean setup keeps `Skill`, `Wiki`, and host names as product terms: never translate `Skill` as `기술`.
