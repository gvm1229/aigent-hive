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
  - "repo:.agents/directives/01-behavior.md#sha256:7679dd5603fdfa1104b0017e9fe7c7acb6a81f9e4095233ccddf3db01a325af8"
  - "repo:AGENTS.md#sha256:3be0757d02ee1cb78a8ceebbee5564219ff711d5a6d4465d9318aae61a011778"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:e1e23470bdd37528700da00641cedef9e510c525863c991c9d189ab761e9bde1"
  - "repo:crates/hive-render/src/lib.rs#sha256:d7ac376de1ffbfdf6f04f900fa8b46d78749755ffe9c30e1710be42611353ef7"
  - "repo:harness/directives/00-project-harness.md#sha256:1c86dade7bbc2dcf791b8eda27d68e7759645033c54c885553b69b08cf9da319"
  - "repo:harness/template/AGENTS.md.jinja#sha256:27a80d0856d5f69ed2670eb441068950a42b207d592d9ba892542aa56c99bffc"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:91f5fe71c2f45f7c6b5b47dcada9900f6561572e652689d6cffeed5973bc68c6"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.
