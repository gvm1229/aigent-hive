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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/plans/active/host-neutral-continuation-0.10.0.md#sha256:5c461f0c68bfbf71b762648a546b5e54ec9a66edfd7dd3ac67f1169ff1ee1807"
  - "repo:docs/research/host-neutral-continuation-hooks-0.10-feasibility-2026-08-22.md#sha256:73f86588991a14134009c0bd30503c1215d073a5961c221293036401b2b418e7"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:c37e8cbb4918ef2b6274e4d0cf814c9157b324ad"
status: active
---

# Host-neutral 연속 실행 gate

- 실행 주체: Host 소유 Goal 또는 task
- 정본: Provider-neutral Markdown run·closure 판정
- `hive run closure`: plan·status criterion 일치 확인·pending·blocked·closure digest 반환
- Stop adapter: 새 run revision에서 Agent 소유 작업이 남은 경우 1회 nudge
- 종료 허용: 사용자 cancel, `blocked_on_user`, terminal, stale·malformed state, foreign session, 진행 없는 반복 Stop
- Hook mutation: Host Goal·canonical run state 변경 `0건`
- Antigravity local: CLI `1.1.18`의 `/hooks` JSON surface 확인, 설치 hook `0건`, 실제 Stop 차단 미검증
