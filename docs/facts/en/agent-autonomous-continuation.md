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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:193eb021ec2fc367176127ae05ab9dc2baea6097819ddd63de43d9a79ee7cc9a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:b7a409e044561428f1bda83e8daef5a647a63c23a8f1d1426baffb336b63323a"
  - "repo:crates/hive-render/src/lib.rs#sha256:334534e638f8266e8f4aa3b98e1092dd657bbe47f0d7c8ae40b932a64d5b1df9"
  - "repo:harness/directives/00-project-harness.md#sha256:7073d74da76915240b5061818f9a00644872a3b5ff7d84fa55e30dc26c99a50c"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:4c71dbbc592620844bff81765151014d4a657924d03aea602c08bc517e1c8727"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:746a446b6d86020d9536cdbea8bff82d822075338d8f4bbfdaa3202f5dafffd0"
links: [automated-user-handoff, source-development, steering-preservation]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.

An observed usage halt overrides continuation and closure. After exhaustion, the agent sends only a short halt notice and defers checkpoints, knowledge, lint, Git, cleanup and handoff tools until a new request receives fresh allowed usage.
