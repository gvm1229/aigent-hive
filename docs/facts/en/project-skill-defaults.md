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
  - "repo:crates/hive-render/src/lib.rs#sha256:bd58a2ef9a317149e6c76b0d23632f67174e139a9f339770247ffb458e0e9c39"
  - "repo:harness/project-setup/skill-suites.yml#sha256:58ed9002b60fe1b9907c6d9b2cfb1d9846afb5a5b551c98c2c08024968e333c1"
  - "repo:schemas/setup-answers.schema.json#sha256:7f3ec163942955be5ad16c177eafbf648cf209e52c701e6334c651b521e24624"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 27 built-ins. It excludes user-setup, project-setup, custom-subagent-create and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes three knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
