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
  - "repo:.agents/directives/01-behavior.md#sha256:dd66d053a9edd60c2f04e96283f4f95e5429dbf24e6b2d98c025bbf89039d5df"
  - "repo:.agents/directives/04-documentation-state.md#sha256:e941e74431e44442bb5940df43832b72ecfdcc4f3cb4963462ce6ee5ada2a32f"
  - "repo:.agents/directives/06-session-coordination.md#sha256:a24536201b77619549620d88612c186b769e90a774043895370a064779d8d758"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:crates/hive-render/src/lib.rs#sha256:69ebe3eb4fe0f9143725a38b5f9816ac894700102436a42e3a7ae996316b86ac"
  - "repo:docs/archive/plans/foundations/agent-autonomous-continuation.md#sha256:83b8604202102dc5424c63648833d99978d2d415b974e22c7d70fc511f1c5883"
  - "repo:harness/directives/00-project-harness.md#sha256:fb6cb8107a38aa3fe70040d4e730e53190a66ed6047a8e40f55acf811425d87d"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:69b77460d6138cb83ef1b31d8da4075e4d02e4b4213bfb709da0538a1fcc3be8"
  - "repo:tests/fixtures/run/agent-autonomous-continuation.json#sha256:168eb72b79508187e841e5caf25d88f15a86a43b10f0327d7c1ce5a8226aa934"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:0fd5ea87fa377dc584dcfa6ad93ae9ee74eb4e97"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an in-scope inspection, fix, verification, commit,
permitted push, CI observation, or authorized publication remains. This applies to rendered
project guidance and global user guidance in English and Korean. Final closure requires no
agent-owned action; user authority, external evidence, and blocked states carry exact owners and
recovery evidence.
