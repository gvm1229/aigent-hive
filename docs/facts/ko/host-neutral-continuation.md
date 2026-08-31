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
  - "repo:crates/hive-cli/src/run.rs#sha256:7d06c4ec078b4cc6df2004f923e9720b4d5f0406b6badd246aad8220853628d1"
  - "repo:crates/hive-core/src/run.rs#sha256:f9f45d8c48283ce08dbe900387493e268143f6f3b1280dcab7c8e3c358b80103"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:fc19bed8a17b8b8652c37ff518528ada2aec511e163b15c99af90235e6728a82"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:42a2bacfb28c1ee12a73765d027cfb74b7ad786b659d12dbffc098337e09d62c"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
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
