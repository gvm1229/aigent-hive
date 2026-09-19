---
schema_version: 1
pair_id: host-neutral-continuation
topic_slug: host-neutral-continuation
language: ko
counterpart: ../en/host-neutral-continuation.md
title: "Host-neutral 연속 실행 gate"
summary: "Host 소유 Goal·task의 전체 차단 전 모든 미통과 criterion 범위 확인 closure gate"
tags: [hooks, orchestration, v0-10]
aliases: ["Continuation closure gate"]
sources:
  - "repo:crates/hive-cli/src/run.rs#sha256:c82fe6ccb977a438a4c76153c6abaf7bb42bd3fc5419baab2fd4257f362891b0"
  - "repo:crates/hive-core/src/run.rs#sha256:f9f45d8c48283ce08dbe900387493e268143f6f3b1280dcab7c8e3c358b80103"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:b540e5ca68afee2e3947932e9b21bef1c5707cbde322d7c89cd287965609d5cf"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:42a2bacfb28c1ee12a73765d027cfb74b7ad786b659d12dbffc098337e09d62c"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:dd63333a702a7a89585d101d2b9d043ebd0987d8"
status: active
---

# Host-neutral 연속 실행 gate

- 실행 주체: Host 소유 Goal 또는 task
- 정본: Provider-neutral Markdown run·closure 판정
- `hive run closure`: plan·status criterion 일치 확인·pending·blocked·closure digest 반환
- `blocked|usage-limited`: 모든 미통과 criterion의 `blocked_criteria` 범위 일치 필수, partial 범위 거부·write `0건`
- continuation checkpoint: session digest·최대 3회 retry·used attempt·cancel 상태
- Stop adapter: 새 run revision에서 Agent 소유 작업이 남은 경우 1회 nudge
- Hook mutation: Host Goal·task·canonical run state 변경 `0건`
