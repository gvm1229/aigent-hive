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
  - "repo:.agents/directives/01-behavior.md#sha256:9d8adb7c75015fd24df8cb226a16180548c600dc963ee154c0a4af408e9fa48c"
  - "repo:.agents/directives/04-documentation-state.md#sha256:5660c7d72b0bb89f8d105a50d7d3768bcf93d3728d855704df5bfad815744d02"
  - "repo:.agents/directives/06-session-coordination.md#sha256:884fedad85a6bd5c7865b5fc6be9b132c4653abb8d685f26aff621596f6ae48a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9fa9e439ad15ea6a8b5ed7cf6d031595a8979b056dada55360cb32331d9e8355"
  - "repo:crates/hive-render/src/lib.rs#sha256:45cb895735677baf049a54e7875c0ce9eb31b79a044f4b1e78d68af844cde507"
  - "repo:docs/plans/active/agent-autonomous-continuation.md#sha256:83b8604202102dc5424c63648833d99978d2d415b974e22c7d70fc511f1c5883"
  - "repo:harness/directives/00-project-harness.md#sha256:fb6cb8107a38aa3fe70040d4e730e53190a66ed6047a8e40f55acf811425d87d"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9b45162b48b5d6eedcce0d0b6bc2d63613776fc1089a7d2878941897d0d506c5"
  - "repo:tests/conformance/test_phase3_static_contracts.py#sha256:a1480b59d9bdff0b7c5d27c50bf18dcb7a49f56835b872d4d5829580202a9db4"
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
