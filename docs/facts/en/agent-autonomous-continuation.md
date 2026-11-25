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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:a40a0ad017c8287244994186b004c017560a7c30f1f41b722f61180f0fb41d69"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ecb5eb8be00aa5902b1cf2d11d9bcdb51e76ceab68fce047c5463a9fc5920762"
  - "repo:crates/hive-render/src/lib.rs#sha256:3a7521d803fb1db33e44d0b24c33453e17c0cc0f9f674aba814b7649cac4c323"
  - "repo:harness/directives/00-project-harness.md#sha256:2745a3b014401254f4188ce8eff69270453fd3a9fc22e12c0b1ecf285f989559"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:41d5e94d3a19590d65d4bd66cda74777509aa70a5debc128923f3806c7867edc"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:c50423d11ab7057e70696a2e4c532ab33beac1a6bfb3416598aa462c6208e21b"
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
