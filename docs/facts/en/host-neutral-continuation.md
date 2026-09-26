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
  - "repo:crates/hive-cli/src/run.rs#sha256:aa9645f93f49206248e55e76e436da11c6a6b422b9e90130c9515bb988e1342e"
  - "repo:crates/hive-core/src/run.rs#sha256:f9f45d8c48283ce08dbe900387493e268143f6f3b1280dcab7c8e3c358b80103"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:b540e5ca68afee2e3947932e9b21bef1c5707cbde322d7c89cd287965609d5cf"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:df8aa9994a9fa02a4ee782567f646f664d7414ca244aa679e49498a7832b041f"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:0340f8a26d14ebcc134c14e421d605f8022912f8"
status: active
---

# Host-Neutral Continuation Gate

Hive keeps run and closure decisions in provider-neutral Markdown. Read-only `hive run closure`
reports matching plan and status criteria with a closure digest. A `blocked` or `usage-limited`
checkpoint must list every still-unpassed criterion as `blocked_criteria`; a partial list is
rejected without a write. The host owns task execution. Hooks may provide one bounded nudge and
must not mutate host goals, tasks, or canonical run state.
