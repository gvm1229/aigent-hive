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
  - "repo:.agents/directives/01-behavior.md#sha256:94e9c3723758b46a23989ca9264b0951c4e9bf1096c7c5766d39697bf501f7e3"
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2994d811846e136cc1ac82a8d6044ebd8eed238f0ef2f4470a33f248a6ea5ee2"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:ee4aa374f3452d250a376e66c1d1d49ce28c076b95aab8a8bdb9b155e05bca7e"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:crates/hive-render/src/lib.rs#sha256:f68d23e2eb431d0a36193df9c26d59db5fdc8d00afc9dc00bf1588175e486c0c"
  - "repo:harness/directives/00-project-harness.md#sha256:6e45cc7d0b2c675107aa57003f883c0429ca16de40413e90c12c067f20d20f92"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:5a20040c568570b7501cf6715838becb35f24adea960c4f63bd6e245b358ef13"
links: [automated-user-handoff, source-development, steering-preservation]
reviewed_revision: "git:9fc4580a60a40af3161efb32b68726d66b41f294"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.
