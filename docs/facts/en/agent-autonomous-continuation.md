---
schema_version: 1
pair_id: agent-autonomous-continuation
topic_slug: agent-autonomous-continuation
language: en
counterpart: ../ko/agent-autonomous-continuation.md
title: "Agent Autonomous Continuation"
summary: "Independent agent-owned work prevents a whole-goal block; stable release remains explicit-only."
tags: [agent, completion, regression]
aliases: ["No mid-task halt"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:7d8300e65cd3136b350aa96000437faff764cb11af33ed42dedf4c88579448ea"
  - "repo:AGENTS.md#sha256:d1a4541174db15faf38f3c90432fbea8cb4b4da6448bfccce2a7e069982031b6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:85b13d22add18756fa11e29fcc1ebcf84b18d143385991143a8453c29e3d0328"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ab2aaa4dd8d3ec7e90c366a65cf131b6eb2401f1b0b2c95c87d4a6448c7b3bd9"
  - "repo:crates/hive-render/src/lib.rs#sha256:54c93f6fdf51beda50d73eba8e3ea0a06e2441f69f93a8b40440a9d2fc37d767"
  - "repo:harness/directives/00-project-harness.md#sha256:82dd650f61c25f8a2cba930718728052936e10390bd0bbb68e4fed71f14fa520"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:8818a6eb6d47a571477ec7beae8ecb3b7c70610944124bd5aea764e3a960d021"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.
