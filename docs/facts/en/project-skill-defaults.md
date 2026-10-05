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
  - "repo:crates/hive-render/src/lib.rs#sha256:c2abca0c0461baebddc1fe16992eac0e8ec6bdf0dbcf74daf2343a93b9824cea"
  - "repo:harness/project-setup/skill-suites.yml#sha256:c120d17e58acb2d3731c6963ca478a281034952412cc0f3c1b2de43791db0ff5"
  - "repo:schemas/setup-answers.schema.json#sha256:f9b463d937ba88b4ce5e1b9bc904cce5e15a0cfba4c347f9d9a29234ba994146"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 23 built-ins. It excludes user-setup, project-setup, custom-subagent-create, knowledge-transfer and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes five knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
