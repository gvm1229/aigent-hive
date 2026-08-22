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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6ee84b034e6a23171889dc33e8be8f839594edd080168ad7601e8c5fd9e5c9cc"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:786da31401085e9445495aa37defe7cedf781bc8457211a6addd23016c0bf922"
  - "repo:crates/hive-render/src/lib.rs#sha256:2522dab8e855f4ef6a31ae7923c7ab35205f03c6caaefb91896eae1c5e4c75aa"
  - "repo:docs/archive/plans/foundations/agent-autonomous-continuation.md#sha256:83b8604202102dc5424c63648833d99978d2d415b974e22c7d70fc511f1c5883"
  - "repo:harness/directives/00-project-harness.md#sha256:fb6cb8107a38aa3fe70040d4e730e53190a66ed6047a8e40f55acf811425d87d"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:019c1101f7bfb68641ed6686e17690d7469464b7054e28717c9c2b87afd5d423"
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
