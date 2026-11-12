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
  - "repo:crates/hive-render/src/lib.rs#sha256:b6285195847e5b1779bd4a168315a5337eac5885ebacabce45171b22959bf199"
  - "repo:harness/project-setup/skill-suites.yml#sha256:66bb2a743e2b486ec52666b69848b604bcc79bd35d3f741105a88623fc27cf56"
  - "repo:schemas/setup-answers.schema.json#sha256:c4fb1f2da1748a1e087a36d58fb4e63378b2e216e2dfad5bd283e9f711459631"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 32 built-ins. It excludes user-setup, project-setup, custom-subagent-create, knowledge-transfer and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes five knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
