---
schema_version: 1
pair_id: host-neutral-continuation
topic_slug: host-neutral-continuation
language: ko
counterpart: ../en/host-neutral-continuation.md
title: "Host-neutral 연속 실행 gate"
summary: "Host 소유 Goal·task와 bounded optional Stop hook을 결합하는 Hive의 provider-neutral closure gate 계획"
tags: [hooks, orchestration, v0-10]
aliases: ["Continuation closure gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:b911eee77529e280f80a41a9115766d3a9295ffd09cc968962f27542490a2e52"
  - "repo:docs/plans/active/host-neutral-continuation-0.10.0.md#sha256:e45436aa451a03f72ae907733a4c78de02f1dd2b84aad7d3b50dbb75031555bc"
  - "repo:docs/research/host-neutral-continuation-hooks-0.10-feasibility-2026-08-22.md#sha256:eb7425e6f642ba6248cd4675f6376e7dc22cf9b3845ce87bd08b09c9d16ffc70"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:aa0563cae15225cd7aefdaaa2c14346ea503fbd7"
status: active
---

# Host-neutral 연속 실행 gate

- 실행 주체: Host 소유 Goal 또는 task
- 정본: Provider-neutral Markdown run·closure 판정
- Stop adapter: 새 run revision에서 Agent 소유 작업이 남은 경우 1회 nudge
- 종료 허용: 사용자 cancel, `blocked_on_user`, terminal, stale·malformed state, foreign session, 진행 없는 반복 Stop
- Hook mutation: Host Goal·canonical run state 변경 `0건`
