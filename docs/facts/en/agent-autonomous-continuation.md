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
  - "repo:.agents/directives/01-behavior.md#sha256:49c79d8137a1e1d3cdcb06b60fb7e2708e2cf0ae04018a6c9c866ecb0a65a333"
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:42f69c326667ad73522caeadec761a523074a78efcb5caf05a74185acb0fe3ce"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:crates/hive-render/src/lib.rs#sha256:9a8dd35a7cbd20a71c44e5a09330410bf45bdd68de04c623454187d706687449"
  - "repo:harness/directives/00-project-harness.md#sha256:6e45cc7d0b2c675107aa57003f883c0429ca16de40413e90c12c067f20d20f92"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:e6c5137a0c1e61dc0845202cbfa18421238ba0eef7550c08a3dd2f3bdd73cbc5"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:bb177081c93b9d196d9dc3e07acefae9908c7d0e"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.
