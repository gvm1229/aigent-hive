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
  - "repo:README.md#sha256:54a3a6a43e7bbaf6c9c07dab305b6d10ece7cae37fb1d74aae3d8d697ed71de6"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0bae3e86f788d8720c04f151ac37ad8120e5e2e57623c5fe56041abe7c82c594"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:harness/user-setup/catalog.yml#sha256:d55a8ec71cddf4c2cd946850f8c345d58951ae4156b33659d57fe5c4d313f522"
  - "repo:schemas/user-setup.schema.json#sha256:cefe939bedb7f6645fd1851e086def9677e4a2f3f7774258d54dd203cc8b10fc"
links: [global-onboarding, language-consistency]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Global User Contexts

Global setup stores any combination of web development, game development, and general knowledge
work, plus an optional short description. They describe the user, never a project workflow,
implementation choice, delivery priority, or Skill selection.

All built-in Skills are the default; users may instead toggle individual Skills. A saved legacy
single profile migrates to its matching context; a legacy custom profile retains its description.
Korean setup keeps `Skill`, `Wiki`, and host names as product terms: never translate `Skill` as `기술`.
