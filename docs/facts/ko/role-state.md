---
schema_version: 1
pair_id: role-state
topic_slug: role-state
language: ko
counterpart: ../en/role-state.md
title: "Persistent role state"
summary: "Host session을 넘는 role identity·handoff canonical Markdown."
tags: [role, state]
aliases: ["Role lifecycle"]
sources:
  - "repo:docs/architecture/role-lifecycle.md#sha256:da313136e7bc53c8cc85040cb4c8c028461154cb076c7a5a4f3741560d35be55"
links: [run-recovery, skill-routing]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Persistent role state

Canonical Markdown 대상: role identity, assignment, body, handoff. Host session·subagent
process 종료 뒤에도 durable role owner와 continuation data 보존.
