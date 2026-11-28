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
  - "repo:crates/hive-render/src/lib.rs#sha256:db8acd2d00532f6ed23b0adce6e9c8f3a3648c74874cc860eeb3633b20a0e92d"
  - "repo:harness/project-setup/skill-suites.yml#sha256:d2fc0676ba94bcfcb55a8694c21a6f424d8f5f023c96566297a386df3d8024c2"
  - "repo:schemas/setup-answers.schema.json#sha256:0b2610b633e440d8ba534aa0136ea8a28f08e93b9fd80bdf56f7694485d5069a"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 27 built-ins. It excludes user-setup, project-setup, custom-subagent-create and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes three knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
