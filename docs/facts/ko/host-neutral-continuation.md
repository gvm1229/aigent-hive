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
  - "repo:crates/hive-cli/src/run.rs#sha256:7e5e36779280c7e039abb2241266a5531320bfb6fceb8d6a2d82d5c4e77fa1ed"
  - "repo:crates/hive-core/src/run.rs#sha256:61eadfbbb9f6fa834f9c3579d51b78e39688c5c93c6e503aa4aca47d2cd68050"
  - "repo:harness/project-bases/0.16.1/skills/verified-workflow/SKILL.md#sha256:f06e29b9f7738d4ef33c16bd45fea4844e9408b87ee287c611d152d2f6c76cf6"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:76b3ca6dea80f0da390d90e4c0a3e26daac0f4026d1ba56409b6397b002e78cb"
links: [agent-autonomous-continuation, consumer-session-coordination, v0-10-product-scope]
reviewed_revision: "git:cd38b4c24b580a8ef800837a91f88e7a0301111c"
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
