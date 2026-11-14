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
  - "repo:crates/hive-render/src/lib.rs#sha256:29da4ca9b29f58ec853dab54eef3cbfa0119896206ab352335bb1cb05a50b817"
  - "repo:harness/project-setup/skill-suites.yml#sha256:7ee263e954cf1dbba42022fdbdd19f53cafd9064b2345c7c5e0cd0db36186f84"
  - "repo:schemas/setup-answers.schema.json#sha256:99da8a488720b5c0f7891a73108caacfcd8f368fc415ebbf11d20cd09783f737"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 32 built-ins. It excludes user-setup, project-setup, custom-subagent-create, knowledge-transfer and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes five knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
