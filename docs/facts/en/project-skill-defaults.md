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
  - "repo:crates/hive-render/src/lib.rs#sha256:0f57f976119a94d844faa5fbcab16eae37ed838041ffc9d089ebeb5b8a36ae3f"
  - "repo:harness/project-setup/skill-suites.yml#sha256:c120d17e58acb2d3731c6963ca478a281034952412cc0f3c1b2de43791db0ff5"
  - "repo:schemas/setup-answers.schema.json#sha256:fc6ee25893e4aeb8446d1b1c8b40db6fcf00c4685a47d53c2301b2690df50a27"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:0ee29cc7a10c6786f5d325569f85182e654cd4b1"
status: active
---

# Project Skill Defaults

The daily-work project suite explicitly lists 23 built-ins. It excludes user-setup, project-setup, custom-subagent-create, knowledge-transfer and project-transition. Project selection can be customized or empty; user-setup remains global-only. Wiki-off removes five knowledge Skills. Policy 1 proposes the new default once for legacy installs and preserves later choices. Selected Skills support request routing without granting mutation or publication authority.
