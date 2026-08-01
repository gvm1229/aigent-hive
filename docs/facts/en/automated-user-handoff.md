---
schema_version: 1
pair_id: automated-user-handoff
topic_slug: automated-user-handoff
language: en
counterpart: ../ko/automated-user-handoff.md
title: "Automated Work Before User Handoff"
summary: "Hive finishes safe automatable work before handing concise user-owned steps to the maintainer."
tags: [automation, behavior, handoff]
aliases: ["todo handoff", "user-owned steps"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:69cad89a5e857e404f6d51106a8688623afd6d3ad1613ddc5a326ab7b998bb30"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c3cc02dcd02afddbd583d51bd02bc113dc283a17e8244587e0bbf832450dd823"
  - "repo:harness/template/AGENTS.md.jinja#sha256:71eeaf7aff5e21b8a7cf764daf6060cb44954f14218370585c3d72a6f25f14c7"
links: [language-consistency, source-development]
reviewed_revision: "git:19eda4d7ef87fe3122c14c455df07758c3dc6ff1"
status: active
---

# Automated Work Before User Handoff

Source agents and installed consumer harnesses finish every safe, authorized,
in-scope automatable action before presenting pending work. The handoff contains
only user-owned steps, each with the exact location or command, expected result
or evidence, and the reason user authority is required. Failures and impossible
actions are separated with their cause and recovery path. Acceptance requires
matching source and consumer projections plus regression tests. Origin: the
maintainer requested actionable guidance instead of a bare pending-task list.
