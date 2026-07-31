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
  - "repo:.agents/directives/01-behavior.md#sha256:2532c785b59f23a099b9e4a6eb71798f696dc4b79103600cf7c245582afa9f26"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c5cb31b7cf39c02be926e38ee529e023aabe45870b84a75b711f4f84c424e282"
  - "repo:harness/template/AGENTS.md.jinja#sha256:e9545c960f609ad7369e2d5e0cc9f48f79fdc7cd20836cf6199f19eb4ca4f301"
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
