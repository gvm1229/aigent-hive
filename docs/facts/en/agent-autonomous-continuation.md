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
  - "repo:.agents/directives/01-behavior.md#sha256:20905d49494df815461b4e9ffe6df89ee33ccb774510da2cfa10c98f0508b077"
  - "repo:AGENTS.md#sha256:24ab31ad8304747818fb45fe3b80255a54d5f615195181f3bb7dd6f4188b0702"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:359e033f6bad6a6145820efb0a079a6643d4774a6d9b8e1b560d9d4e156df5be"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:64e7ee1eb9aaafd399fe971ca35e5df6aee68285029a9b84fa6b928a3324ffdc"
  - "repo:crates/hive-render/src/lib.rs#sha256:0649bdd034cd1904a2775ccda92b04ba04a6d2fa1dfb246b093794e4f5debc7b"
  - "repo:harness/directives/00-project-harness.md#sha256:96db57717e3d03cd0b8ccb28fb5fb4a4dbd8e3ab594c98893f736863a9364415"
  - "repo:harness/template/AGENTS.md.jinja#sha256:f1170037b949896332fdb95f058fde810a00b0474b423e054899a74a5da3b200"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:be88ce1d1993eefdaafe3d2499d855f8a41a73a29cfcc7dfba22864a9e8739a0"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:64125db02505a9a696e870d23fa54feb125b8093"
status: active
---

# Agent Autonomous Continuation

Source and consumer agents must continue while an independent in-scope action remains. A partial
host, fixture, or external-evidence failure stays with its criterion and cannot block a whole Goal
or task. Stable tag, protected-branch integration, publication, and installation require explicit
authorization of the named stable version in the current request.
