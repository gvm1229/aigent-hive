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
  - "repo:crates/hive-cli/src/run.rs#sha256:4fdb81d3893e68681945090bedccfaa3b4a74f88593ec169a4ec0ef10c19fbc9"
  - "repo:crates/hive-core/src/run.rs#sha256:f9f45d8c48283ce08dbe900387493e268143f6f3b1280dcab7c8e3c358b80103"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:f06e29b9f7738d4ef33c16bd45fea4844e9408b87ee287c611d152d2f6c76cf6"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:1e472bb4458467341a9cc571e90f2679f6272f30692d84c0bc99b895ef2dd42c"
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
