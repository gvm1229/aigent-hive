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
  - "repo:.agents/directives/01-behavior.md#sha256:a3fad4148b713fa44c79c4906c297e621a529798785f9b916d67fc0aeff8b4e5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:a722397099348116c5c43cdee8753249b8c8f293be480f23d298f2d2a2eeb04b"
  - "repo:harness/template/AGENTS.md.jinja#sha256:bb858b1021be8b3fd9fc282820a34a4e923dea6a47e01bdddcf9745510c1381d"
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
