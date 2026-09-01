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
  - "repo:crates/hive-cli/src/run.rs#sha256:7d06c4ec078b4cc6df2004f923e9720b4d5f0406b6badd246aad8220853628d1"
  - "repo:crates/hive-core/src/run.rs#sha256:f9f45d8c48283ce08dbe900387493e268143f6f3b1280dcab7c8e3c358b80103"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:fc19bed8a17b8b8652c37ff518528ada2aec511e163b15c99af90235e6728a82"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:42a2bacfb28c1ee12a73765d027cfb74b7ad786b659d12dbffc098337e09d62c"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Host-Neutral Continuation Gate

Hive keeps run and closure decisions in provider-neutral Markdown. Read-only `hive run closure`
reports matching plan and status criteria with a closure digest. A `blocked` or `usage-limited`
checkpoint must list every still-unpassed criterion as `blocked_criteria`; a partial list is
rejected without a write. The host owns task execution. Hooks may provide one bounded nudge and
must not mutate host goals, tasks, or canonical run state.
