---
schema_version: 1
pair_id: role-state
topic_slug: role-state
language: en
counterpart: ../ko/role-state.md
title: "Persistent Role State"
summary: "Role identity and handoff remain canonical Markdown across host sessions."
tags: [role, state]
aliases: ["Role lifecycle"]
sources:
  - "repo:docs/architecture/role-lifecycle.md#sha256:a1bf8a20f8836822634c315f5cfed276232a99efb5346bae7fb3f15f593ab535"
links: [run-recovery, skill-routing]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# Persistent Role State

Role identity, assignment, body, and handoff are canonical Markdown. Host sessions and
subagent processes may end without erasing the durable role owner or continuation data.
