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
  - "repo:README.md#sha256:05c2f90b0096663c3c8ded051eec7d7632df4b99442d90a77faca4f26b5bf80b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:54da258502aa42f21df3882984fa40a697e589b84209a809005996907a4f4c82"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:harness/user-setup/catalog.yml#sha256:e1827279bb1021ca44a14d543d733b142be8c3a11462893a10405f0d683cf7f0"
  - "repo:schemas/user-setup.schema.json#sha256:0990627df1cd626e265c6f7febd4f5ee0711fdfc8a0ebaca23607c140d0ad87c"
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
