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
  - "repo:.agents/directives/01-behavior.md#sha256:6587e8fa5aa274f2c981ad28c062d3c8c388e351440c04663a50122570986976"
  - "repo:.agents/directives/04-documentation-state.md#sha256:44913afc655f527245720594f16a92c87061abfc28280f1a2834ad328b336be5"
  - "repo:.agents/directives/06-session-coordination.md#sha256:884fedad85a6bd5c7865b5fc6be9b132c4653abb8d685f26aff621596f6ae48a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9fa9e439ad15ea6a8b5ed7cf6d031595a8979b056dada55360cb32331d9e8355"
  - "repo:crates/hive-render/src/lib.rs#sha256:9d5ae48c8c77e11cc59db83c53a387d2e85329e4508b66558b40c55a419f0534"
  - "repo:docs/plans/active/agent-autonomous-continuation.md#sha256:83b8604202102dc5424c63648833d99978d2d415b974e22c7d70fc511f1c5883"
  - "repo:harness/directives/00-project-harness.md#sha256:fb6cb8107a38aa3fe70040d4e730e53190a66ed6047a8e40f55acf811425d87d"
  - "repo:harness/template/AGENTS.md.jinja#sha256:63e361ae2218f00f6a22f5e192c25a5c3bcddc21d51f61006f74a0459b636a38"
  - "repo:tests/conformance/test_phase3_static_contracts.py#sha256:71224482ebc778130104504faf981d59b25e5c9790e7dec89f358b347fcd2e53"
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
