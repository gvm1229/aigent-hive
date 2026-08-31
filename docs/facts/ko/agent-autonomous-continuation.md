---
schema_version: 1
pair_id: agent-autonomous-continuation
topic_slug: agent-autonomous-continuation
language: ko
counterpart: ../en/agent-autonomous-continuation.md
title: "Agent 자율 실행 지속"
summary: "독립 Agent 소유 작업 잔존 상태: 전체 Goal·task 차단 금지와 안정판 명시 승인"
tags: [agent, completion, regression]
aliases: ["중간 종료 방지"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:3a8450ff3e496f4e6bafc7b8d10cdd9fe38f15932b465d131a69ca0bdf9ef2f3"
  - "repo:AGENTS.md#sha256:d1a4541174db15faf38f3c90432fbea8cb4b4da6448bfccce2a7e069982031b6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:1518c1b9ac4f68d114a59603a490491221b0459e36137fb380d2c247f9e1ab1a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:8ffb5878b47033d6756f32b270f43b5c8df19243df499d0668ba31678b35672d"
  - "repo:crates/hive-render/src/lib.rs#sha256:58d45eb16a719523947a4ad6b50bc225a757aa2ca800ec95dbf957b74325803d"
  - "repo:harness/directives/00-project-harness.md#sha256:b01acbf296d63e415b06c237561494b73ce632174925f9ad5fd4e2dfb6f6a9e4"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:5e5ce3f56aa6868e8e6195f48cc2c22936d642c70a0f466fe7081108f5ebb28e"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지
