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
  - "repo:.agents/directives/01-behavior.md#sha256:cf2be8b3eb6423c8cc3098ad96553544e37d8f13edc8c6cdc48262e39d82c662"
  - "repo:AGENTS.md#sha256:a541ce4a4aff8b3dcef37d728bef434073900bfed4cc45dc6c0384346b7308d1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:0360d325822c7b7407e12076c809b9c7a17fd189badcae469177ebd1acabed83"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a1c88321abacefb8eac2a6643d71421ee8f1283cce87bd3f41cdd201d18691c3"
  - "repo:crates/hive-render/src/lib.rs#sha256:421b619b2e7990bd2ceea9b2323231b60cb085f60fa1be8c0e62704f738ae7e4"
  - "repo:harness/directives/00-project-harness.md#sha256:8bced45c182a1fe5810cb5cd13e918e1b6dd50e7a24054bcf22a86d148243c8a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:c256f2dfdf227d99a56138d6099b0eaf99c12a8d4e49e831e45b553b5db142a9"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:ecbe2bbaa6391edc69f3490ca00dd14b9fc202e9ea03e7880939f6146ad97cbd"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:5257f45"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.
