---
schema_version: 1
pair_id: agent-autonomous-continuation
topic_slug: agent-autonomous-continuation
language: en
counterpart: ../ko/agent-autonomous-continuation.md
title: "Agent Autonomous Continuation"
summary: "Usage halts take priority; otherwise independent work continues and stable release remains explicit-only."
tags: [agent, completion, regression]
aliases: ["No premature closure"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:AGENTS.md#sha256:0588c67a687a50d00387ab18e2dda2d79a8bcf3344b67f63849cd5f8cab0dabb"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2527e7ceccba9293bc6f0ca7a7ca53ae986ec82066f1f4c62abc8426e4f18ab8"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:c1b10c7785570de5b92f85ef3a2b6a9f43efe12e03e008cc0950bbf156f00819"
  - "repo:crates/hive-render/src/lib.rs#sha256:4b07b57fefd87c090dc12db47a756f0f5556545f6316c879bf8e0a67f53190b4"
  - "repo:harness/directives/00-project-harness.md#sha256:5d4e8b3c34a968b86735f59d449b194021c16ff8e9b4741b4c3ccaf130aa94b6"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:4c71dbbc592620844bff81765151014d4a657924d03aea602c08bc517e1c8727"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:746a446b6d86020d9536cdbea8bff82d822075338d8f4bbfdaa3202f5dafffd0"
links: [automated-user-handoff, source-development, steering-preservation]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.

An observed usage halt overrides continuation and closure. After exhaustion, the agent sends only a short halt notice and defers checkpoints, knowledge, lint, Git, cleanup and handoff tools until a new request receives fresh allowed usage.
