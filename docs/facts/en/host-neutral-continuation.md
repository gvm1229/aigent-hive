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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/plans/active/host-neutral-continuation-0.10.0.md#sha256:5c461f0c68bfbf71b762648a546b5e54ec9a66edfd7dd3ac67f1169ff1ee1807"
  - "repo:docs/research/host-neutral-continuation-hooks-0.10-feasibility-2026-08-22.md#sha256:73f86588991a14134009c0bd30503c1215d073a5961c221293036401b2b418e7"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:c37e8cbb4918ef2b6274e4d0cf814c9157b324ad"
status: active
---

# Host-Neutral Continuation Gate

Hive keeps the run and closure decision in provider-neutral Markdown. Read-only `hive run closure`
checks matching plan and status criteria and returns pending or blocked work with a closure digest.
A host-owned goal or task continues execution. An optional host adapter can block Stop once for a new run revision when
verified agent-owned work remains. User cancellation, blocked-on-user state, terminal state,
stale or malformed state, foreign sessions, and repeated no-progress stops always permit exit.
Hooks never mutate the host goal or canonical run state.
Antigravity CLI `1.1.18` locally exposes the read-only `/hooks` JSON surface; no Stop hook is
currently installed, so actual continuation blocking remains unverified.
