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
  - "repo:.agents/directives/01-behavior.md#sha256:7679dd5603fdfa1104b0017e9fe7c7acb6a81f9e4095233ccddf3db01a325af8"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6d2138ea9d68803f4447e7295cc08fcf5b548c23df8b2d1e5826c0aa46bff668"
  - "repo:harness/template/AGENTS.md.jinja#sha256:27a80d0856d5f69ed2670eb441068950a42b207d592d9ba892542aa56c99bffc"
links: [language-consistency, source-development]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
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
