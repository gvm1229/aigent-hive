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
  - "repo:.agents/directives/01-behavior.md#sha256:352a41e8879f9993888a178725a2d2b0b8dbae5f8c1edf2cdeeb5e3045cbc2a9"
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:e3f955e48e354aaac598a5237679689fb7c827366574d9330bc6dd4b661735ea"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:4795665ae984b818ed5781b43884bec10088ddb64287465dcbfafbed81b965a0"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:crates/hive-render/src/lib.rs#sha256:62174ca2ea76cf5c379e1638e3ef2c33cc332f3ae312f03a983711f9abe1a1d5"
  - "repo:harness/directives/00-project-harness.md#sha256:ee4f9721824c4caad76c21b7d43e04f708637f3df96a095bf8058ef46b122ace"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:461be3769de1d76f7dee1aefc4f2cae3b360a339f40b41ef13cff404f571b2dd"
links: [automated-user-handoff, source-development, steering-preservation]
reviewed_revision: "git:29e5241e0983e13d222703731533b43409327e6a"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.
