---
schema_version: 1
pair_id: agent-directive-ownership
topic_slug: agent-directive-ownership
language: en
counterpart: ../ko/agent-directive-ownership.md
title: "Agent Directive Ownership"
summary: "Hive routes each rule family to one canonical directive, keeps unspecified development on the active version, and verifies size, route, projection, and duplicate-rule budgets."
tags: [directives, routing, v0-10]
aliases: ["Directive optimization"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:.agents/directives/04-documentation-state.md#sha256:8125c0cdce9daeffb4f2c2b4ad258d4b63a0ff363429bfebb4f4d5852532ff91"
  - "repo:AGENTS.md#sha256:0588c67a687a50d00387ab18e2dda2d79a8bcf3344b67f63849cd5f8cab0dabb"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:db0d5c52f75c5ce82354cde25673a5ff701bf99b3458f4f311b7844555b0df05"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
  - "repo:scripts/check-agent-directives.py#sha256:d0520ff0a53524541cff7965258c89e2a2a2d03df1066312d854ac1ced41e5f9"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:e032bd9f8a1e404cf473dfd78982c9f7780abb13"
status: active
---

# Agent Directive Ownership

Source and consumer `AGENTS.md` are small routers. Each detailed rule family has one canonical
owner; generated entrypoints keep approved summaries. The static gate checks byte budgets, routes,
projection parity and non-allowlisted normalized duplicates. Historical project/user bases stay
immutable. Stable publication needs named-version approval. Without a newly named version, use
the active plan's product version and next numbered public test; never invent a later destination.
Durable maintainer rules for repo agent behavior belong in `.agents/directives/`, or the owning
plan for plan-scoped rules, never only in host-private memory (maintainer, 2026-11-12).
