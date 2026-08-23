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
  - "repo:.agents/directives/01-behavior.md#sha256:20905d49494df815461b4e9ffe6df89ee33ccb774510da2cfa10c98f0508b077"
  - "repo:AGENTS.md#sha256:d8fe84d5fe9bf291465651087a79135880c9b6f17e284e65a4eeb0891d851f2f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:85b13d22add18756fa11e29fcc1ebcf84b18d143385991143a8453c29e3d0328"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:64e7ee1eb9aaafd399fe971ca35e5df6aee68285029a9b84fa6b928a3324ffdc"
  - "repo:crates/hive-render/src/lib.rs#sha256:54c93f6fdf51beda50d73eba8e3ea0a06e2441f69f93a8b40440a9d2fc37d767"
  - "repo:harness/directives/00-project-harness.md#sha256:82dd650f61c25f8a2cba930718728052936e10390bd0bbb68e4fed71f14fa520"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:1cb63b7c7e7513d2c864b8ec066c59dae2acd99460c924ffb22bfdfeb52de24e"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지
