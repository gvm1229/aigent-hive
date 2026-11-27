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
  - "repo:crates/hive-render/src/lib.rs#sha256:95986fc182da052aa80fb7d2e57dae3d1b65cb3a8f9226932ca2914e32050132"
  - "repo:harness/project-setup/skill-suites.yml#sha256:c2144a8477259673cffbb51e5cd99143adca435ac583576b74c9640f5d0f2297"
  - "repo:schemas/setup-answers.schema.json#sha256:1a14afc5c389eaad085a2daedfa0c8e14700fdbaabd8f81a779634c793d51895"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 27 built-ins. It excludes user-setup, project-setup, custom-subagent-create and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes three knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
