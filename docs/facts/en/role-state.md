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
  - "repo:docs/architecture/role-lifecycle.md#sha256:da313136e7bc53c8cc85040cb4c8c028461154cb076c7a5a4f3741560d35be55"
links: [run-recovery, skill-routing]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Persistent Role State

Role identity, assignment, body, and handoff are canonical Markdown. Host sessions and
subagent processes may end without erasing the durable role owner or continuation data.
