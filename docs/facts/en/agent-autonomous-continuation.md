---
schema_version: 1
pair_id: agent-autonomous-continuation
topic_slug: agent-autonomous-continuation
language: en
counterpart: ../ko/agent-autonomous-continuation.md
title: "Agent Autonomous Continuation"
summary: "A task with agent-owned work remains active; a progress report is never task closure."
tags: [agent, completion, regression]
aliases: ["No mid-task halt"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:20c7359fc81cde6dfb49abe8782a7d41b29e534422b035c85ca71263b9d0c00e"
  - "repo:.agents/directives/04-documentation-state.md#sha256:2b1909a619ca2b270dd049df9ad91f892f6fd2734e97e6869c421fe9c5a75090"
  - "repo:.agents/directives/06-session-coordination.md#sha256:884fedad85a6bd5c7865b5fc6be9b132c4653abb8d685f26aff621596f6ae48a"
  - "repo:docs/plans/active/agent-autonomous-continuation.md#sha256:c0d222d2a3e2853fce14a5b93a053c1aa5c69e13f628201ac25e050741d3a1d1"
  - "repo:tests/conformance/test_phase3_static_contracts.py#sha256:610a4e8b4c6e511a972ee1033c74219c076ed9f2977cc8e1e7c79590e1ec3821"
  - "repo:tests/fixtures/agent-autonomous-continuation.json#sha256:168eb72b79508187e841e5caf25d88f15a86a43b10f0327d7c1ce5a8226aa934"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:b8e8ea1d68fc2b37a37b07d5d287c3b40c48edf8"
status: active
---

# Agent Autonomous Continuation

An agent must continue while an in-scope fix, verification, push, CI observation, or authorized
publication remains. Final closure requires no agent-owned action; user authority, external
evidence, and blocked states carry exact owners and recovery evidence.
