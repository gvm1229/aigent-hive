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
  - "repo:.agents/directives/01-behavior.md#sha256:a78fc02202dc5c3b934e28924dd86660d297151f4905606dc7a26f2179083eaa"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:e21faccc9dae23d7522de433e345890509ce8d742fa8fe6a375f0892e35713db"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9e5694a62099d262872bd6e1f167d839d9eb3f51c3d6cdfd4884656350cc0ec4"
links: [language-consistency, source-development]
reviewed_revision: "git:bd6d9249b8641590269d32deb97d13b2816ba75e"
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
