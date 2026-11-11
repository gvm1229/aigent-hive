---
schema_version: 1
pair_id: project-skill-defaults
topic_slug: project-skill-defaults
language: en
counterpart: ../ko/project-skill-defaults.md
title: "Project Skill Defaults"
summary: "Project Skill Defaults"
tags: [directive, project, skill]
aliases: []
sources:
  - "repo:crates/hive-render/src/lib.rs#sha256:caf624451aee9c18bc13ca0b33b0428ff33455856d3e20d6417e1b5c2fa35943"
  - "repo:harness/project-setup/skill-suites.yml#sha256:c120d17e58acb2d3731c6963ca478a281034952412cc0f3c1b2de43791db0ff5"
  - "repo:schemas/setup-answers.schema.json#sha256:bb77a1a06c20871299ef83a384142e137e272790cc2c648628cd1098f7faeb9e"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 23 built-ins. It excludes user-setup, project-setup, custom-subagent-create, knowledge-transfer and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes five knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
