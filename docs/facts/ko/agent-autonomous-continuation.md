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
  - "repo:AGENTS.md#sha256:24ab31ad8304747818fb45fe3b80255a54d5f615195181f3bb7dd6f4188b0702"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:359e033f6bad6a6145820efb0a079a6643d4774a6d9b8e1b560d9d4e156df5be"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:64e7ee1eb9aaafd399fe971ca35e5df6aee68285029a9b84fa6b928a3324ffdc"
  - "repo:crates/hive-render/src/lib.rs#sha256:0649bdd034cd1904a2775ccda92b04ba04a6d2fa1dfb246b093794e4f5debc7b"
  - "repo:harness/directives/00-project-harness.md#sha256:ba4308f93b28bb1920c355c43d7a632fee4e87a80434abdc746150efd4b8e209"
  - "repo:harness/template/AGENTS.md.jinja#sha256:b11663ebd662eb679c11cf115223c5eb47ab762ccd1c26966e83d594b403b67b"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:be88ce1d1993eefdaafe3d2499d855f8a41a73a29cfcc7dfba22864a9e8739a0"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:838842805e453e0508d054e4aa67d7a59b3aa53f"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지
