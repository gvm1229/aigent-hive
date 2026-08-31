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
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:2a7c02ca89bc80f95574e9c6147af3d634bcfd3c40f395a554a225175bf09d91"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:a6aea1ed5b977bc818bace5c9d712d2da01328f59753e9b93136c17b1a8f24d3"
  - "repo:harness/user-setup/catalog.yml#sha256:fed2aefc7efa52c28bb05c5b069ad4c4fbeec30b805fff7b84d00285fca18ea4"
  - "repo:schemas/user-setup.schema.json#sha256:daee52c6535601606bc39d67800ed2e6ad248828ac73383cc7d8ded015c95652"
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
