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
  - "repo:.agents/directives/01-behavior.md#sha256:42bbd59e702cdce48ac6396d4c5a2f3a9b7574cd99272e22f3279c00b041cba4"
  - "repo:.agents/directives/04-documentation-state.md#sha256:44913afc655f527245720594f16a92c87061abfc28280f1a2834ad328b336be5"
  - "repo:.agents/directives/06-session-coordination.md#sha256:a24536201b77619549620d88612c186b769e90a774043895370a064779d8d758"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:crates/hive-render/src/lib.rs#sha256:019c4b9187834d210c659a1ade13f9a30d5b04c45088e5184e04d0340797712e"
  - "repo:docs/plans/active/agent-autonomous-continuation.md#sha256:83b8604202102dc5424c63648833d99978d2d415b974e22c7d70fc511f1c5883"
  - "repo:harness/directives/00-project-harness.md#sha256:fb6cb8107a38aa3fe70040d4e730e53190a66ed6047a8e40f55acf811425d87d"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
  - "repo:tests/conformance/test_phase3_static_contracts.py#sha256:649c12bd367917808f69ad28355a054481d30f06ca84f3a44b984cd339f176db"
  - "repo:tests/fixtures/agent-autonomous-continuation.json#sha256:168eb72b79508187e841e5caf25d88f15a86a43b10f0327d7c1ce5a8226aa934"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:9a125333ed070140b3773462d895684cba62fe6b"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an in-scope inspection, fix, verification, commit,
permitted push, CI observation, or authorized publication remains. This applies to rendered
project guidance and global user guidance in English and Korean. Final closure requires no
agent-owned action; user authority, external evidence, and blocked states carry exact owners and
recovery evidence.
