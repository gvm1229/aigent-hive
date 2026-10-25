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
  - "repo:README.md#sha256:049596fcbb0b062c936adef3eccc3fb154e407778a1c98bbfef6da20314fd07f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9cf945c0072f953b590757cffec4fbda47b68ac0145cb4b573b220aa6ddc4d45"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:b33ed5b0aa3819b3607100012f444faa6af82f7f1363c2ca492cb7eaf81bb1df"
  - "repo:harness/user-setup/catalog.yml#sha256:fed2aefc7efa52c28bb05c5b069ad4c4fbeec30b805fff7b84d00285fca18ea4"
  - "repo:schemas/user-setup.schema.json#sha256:4f9c5f59802bc964cfdc41636c32e7e71929c52d64a0291babba948906567a1f"
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
