---
schema_version: 1
pair_id: host-neutral-continuation
topic_slug: host-neutral-continuation
language: en
counterpart: ../ko/host-neutral-continuation.md
title: "Host-Neutral Continuation Gate"
summary: "Host-owned goals or tasks use a bounded closure gate; a whole block requires coverage of every unpassed criterion."
tags: [hooks, orchestration, v0-10]
aliases: ["Continuation closure gate"]
sources:
  - "repo:crates/hive-cli/src/run.rs#sha256:7e5e36779280c7e039abb2241266a5531320bfb6fceb8d6a2d82d5c4e77fa1ed"
  - "repo:crates/hive-core/src/run.rs#sha256:61eadfbbb9f6fa834f9c3579d51b78e39688c5c93c6e503aa4aca47d2cd68050"
  - "repo:harness/project-bases/0.16.1/skills/verified-workflow/SKILL.md#sha256:f06e29b9f7738d4ef33c16bd45fea4844e9408b87ee287c611d152d2f6c76cf6"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:76b3ca6dea80f0da390d90e4c0a3e26daac0f4026d1ba56409b6397b002e78cb"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:cd38b4c24b580a8ef800837a91f88e7a0301111c"
status: active
---

# Host-Neutral Continuation Gate

Hive keeps run and closure decisions in provider-neutral Markdown. Read-only `hive run closure`
reports matching plan and status criteria with a closure digest. A `blocked` or `usage-limited`
checkpoint must list every still-unpassed criterion as `blocked_criteria`; a partial list is
rejected without a write. The host owns task execution. Hooks may provide one bounded nudge and
must not mutate host goals, tasks, or canonical run state.
