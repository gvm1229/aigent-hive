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
  - "repo:.agents/directives/01-behavior.md#sha256:b84d45bd3fb28b67581ff024fdfb8d0aaf0dfbbda580f712c96d1228685dadbb"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:35f3cace4b6297a298b8b59db208b3d8ecfd82331758fb6bd34dd1ec03aa8ec7"
  - "repo:harness/template/AGENTS.md.jinja#sha256:070d97440343d699565448c239efb55c905df79119df289525d41edc6e81581f"
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
