---
schema_version: 1
pair_id: host-neutral-continuation
topic_slug: host-neutral-continuation
language: en
counterpart: ../ko/host-neutral-continuation.md
title: "Host-Neutral Continuation Gate"
summary: "Hive plans a host-neutral closure gate with host-owned goals or tasks and bounded optional Stop hooks."
tags: [hooks, orchestration, v0-10]
aliases: ["Continuation closure gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:b911eee77529e280f80a41a9115766d3a9295ffd09cc968962f27542490a2e52"
  - "repo:docs/plans/active/host-neutral-continuation-0.10.0.md#sha256:e45436aa451a03f72ae907733a4c78de02f1dd2b84aad7d3b50dbb75031555bc"
  - "repo:docs/research/host-neutral-continuation-hooks-0.10-feasibility-2026-08-22.md#sha256:73f86588991a14134009c0bd30503c1215d073a5961c221293036401b2b418e7"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:0c72846f0690380e013e63f2bbda707105afeb92"
status: active
---

# Host-Neutral Continuation Gate

Hive keeps the run and closure decision in provider-neutral Markdown. A host-owned goal or task
continues execution. An optional host adapter can block Stop once for a new run revision when
verified agent-owned work remains. User cancellation, blocked-on-user state, terminal state,
stale or malformed state, foreign sessions, and repeated no-progress stops always permit exit.
Hooks never mutate the host goal or canonical run state.
Antigravity CLI `1.1.18` locally exposes the read-only `/hooks` JSON surface; no Stop hook is
currently installed, so actual continuation blocking remains unverified.
