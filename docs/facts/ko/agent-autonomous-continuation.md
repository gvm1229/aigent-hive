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
  - "repo:.agents/directives/01-behavior.md#sha256:49c79d8137a1e1d3cdcb06b60fb7e2708e2cf0ae04018a6c9c866ecb0a65a333"
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6da6fa6180feb983e71a9be4915ff93a12cc1bfefae6016bad026f5c94b0238e"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6f0d42adfc0aa7cf6e199898d950a7219e0bc3b3c78ef1bbc5b822d742e142dd"
  - "repo:crates/hive-render/src/lib.rs#sha256:9cf9801d2a43b4db725070b7877cfe3b084d7fb5ee37cd8220e644595ae57cce"
  - "repo:harness/directives/00-project-harness.md#sha256:0852a921da7c0b6eafb5e47c95192f799405bb49f41706c2e77e61a632e53094"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:e6c5137a0c1e61dc0845202cbfa18421238ba0eef7550c08a3dd2f3bdd73cbc5"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:bb177081c93b9d196d9dc3e07acefae9908c7d0e"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지
