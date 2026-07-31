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
  - "repo:.agents/directives/01-behavior.md#sha256:d59f86031a7bb6f889eeaa00598794fdd2f73375da7d03cdb6a5b49d4884dc0f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:18b30d35eef4e0db6e2dce0caef804a3918648a0940fdb189c74a049d59e5f73"
  - "repo:harness/template/AGENTS.md.jinja#sha256:6198d9b0380ee4e46d44a6aab9ea759c0080690e3353a9309da1a12c5b1939c2"
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
