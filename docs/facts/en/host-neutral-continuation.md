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
  - "repo:docs/plans/active/host-neutral-continuation-0.10.0.md#sha256:e20f89db8c8f9d41757c77b8700e158afb0de7472f210494ce7054dada6e9e1a"
  - "repo:docs/research/host-neutral-continuation-hooks-0.10-feasibility-2026-08-22.md#sha256:73f86588991a14134009c0bd30503c1215d073a5961c221293036401b2b418e7"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:8025085afdc50774e309906e4754741348c31c84"
status: active
---

# Host-Neutral Continuation Gate

Hive keeps run and closure decisions in provider-neutral Markdown. Read-only `hive run closure`
reports matching plan and status criteria with a closure digest. An optional checkpoint records a
session digest, up to three retries, used attempts, and cancellation; legacy runs stay fail-open.
The host owns task execution. Hooks must not mutate the host goal or canonical run state.
Antigravity CLI `1.1.18` exposes read-only `/hooks`; actual Stop blocking remains unverified.
