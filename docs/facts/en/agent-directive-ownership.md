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
  - "repo:.agents/directives/04-documentation-state.md#sha256:468031856977484a5c5c39e1af40f4efbe7c953ded7bfc6fa7e4bdabcbc00d82"
  - "repo:AGENTS.md#sha256:0588c67a687a50d00387ab18e2dda2d79a8bcf3344b67f63849cd5f8cab0dabb"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:db0d5c52f75c5ce82354cde25673a5ff701bf99b3458f4f311b7844555b0df05"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
  - "repo:scripts/check-agent-directives.py#sha256:b575636bcb279f1839fbd57c0aec697da34cd4cdc4709ca66e2bc6e122442b00"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
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
