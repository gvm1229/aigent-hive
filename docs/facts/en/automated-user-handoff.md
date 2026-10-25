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
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2fade132e2fe4e60b3f7b70e089d5deaa145a25e2976bdfdb62a58a82fa86192"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:4c71dbbc592620844bff81765151014d4a657924d03aea602c08bc517e1c8727"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
links: [language-consistency, source-development]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
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

Usage exhaustion also defers handoff writes and cleanup. The existing guard receipt is retained without another tool call. A short halt notice is not a completion claim.
