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
  - "repo:crates/hive-render/src/lib.rs#sha256:3a7521d803fb1db33e44d0b24c33453e17c0cc0f9f674aba814b7649cac4c323"
  - "repo:harness/project-setup/skill-suites.yml#sha256:31e7c19ef5cc7b4edf9646f07fa81bbcbc3a37a39b9d34ec7fb0b0c26e5159d2"
  - "repo:schemas/setup-answers.schema.json#sha256:b4661bb62fff062db316f5e3899c11f7e369c8111956b08b2f501be4da9551ff"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 27 built-ins. It excludes user-setup, project-setup, custom-subagent-create and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes three knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
