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
  - "repo:docs/architecture/role-lifecycle.md#sha256:a1bf8a20f8836822634c315f5cfed276232a99efb5346bae7fb3f15f593ab535"
links: [run-recovery, skill-routing]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# Persistent role state

Canonical Markdown 대상: role identity, assignment, body, handoff. Host session·subagent
process 종료 뒤에도 durable role owner와 continuation data 보존.
