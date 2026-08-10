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
  - "repo:.agents/directives/01-behavior.md#sha256:20c7359fc81cde6dfb49abe8782a7d41b29e534422b035c85ca71263b9d0c00e"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:f351f82fe27b3458b25eda8d74f94032206e3ab0a295db901157fa5f14c5e03a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ea732dcaed4b7342f497c6b1268acce269627f07cc1fd596083c30ab300e8fa6"
links: [language-consistency, source-development]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
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
