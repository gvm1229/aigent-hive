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
  - "repo:README.md#sha256:dadb0f1c7cf2873f3af9365c5764def23c0b259f6f3bede16e5d9ee7eb49a637"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:53d45f59e894a17eb3d804249d811b832f5bc1a47f7c3405ef897fd13fb37150"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
  - "repo:harness/user-setup/catalog.yml#sha256:4c3a188bcdb06752531530478c34cedafcaafb259fff00ff8743ed2d2e4c85c0"
  - "repo:schemas/user-setup.schema.json#sha256:988f6c9b16fd5c05e00145f1593aa623ce608e9688227f5a51d85c79af84d9a2"
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
