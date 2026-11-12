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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d47f8ba8f5e3e88783cfc992cb88475d7cf47fa3459baecb434610e7f91c2d45"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:53d45f59e894a17eb3d804249d811b832f5bc1a47f7c3405ef897fd13fb37150"
  - "repo:crates/hive-render/src/lib.rs#sha256:b6285195847e5b1779bd4a168315a5337eac5885ebacabce45171b22959bf199"
  - "repo:harness/directives/00-project-harness.md#sha256:5d4e8b3c34a968b86735f59d449b194021c16ff8e9b4741b4c3ccaf130aa94b6"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:4c71dbbc592620844bff81765151014d4a657924d03aea602c08bc517e1c8727"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:b53ec86c9ad9e878ea0792b85145b9ba7db212242059a881b7c5c3218e1b14b9"
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
