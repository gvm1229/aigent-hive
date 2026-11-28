---
schema_version: 1
pair_id: agent-autonomous-continuation
topic_slug: agent-autonomous-continuation
language: ko
counterpart: ../en/agent-autonomous-continuation.md
title: "Agent 자율 실행 지속"
summary: "사용량 차단 우선, 정상 사용량에서 독립 작업 지속과 안정판 명시 승인"
tags: [agent, completion, regression]
aliases: ["성급한 작업 종료 방지"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:AGENTS.md#sha256:214d39d1c6a871fce91f14dd543ebdfd46c7e61a448f4e9e88077355c5c648fa"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9c5854224d063f07088155f4a61c672587e8bf26c2a67b1c4fb88be9e6a4d4e4"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:04bab46c9c8e3aa4b7491d6ff2e3051d6fa5b4b4a16fc79e46808a6e3e619308"
  - "repo:crates/hive-render/src/lib.rs#sha256:db8acd2d00532f6ed23b0adce6e9c8f3a3648c74874cc860eeb3633b20a0e92d"
  - "repo:harness/directives/00-project-harness.md#sha256:2745a3b014401254f4188ce8eff69270453fd3a9fc22e12c0b1ecf285f989559"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:d263c84680b1b11fc49175ee181955a2b18fd90e819b1929212fd818b176e3fd"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:a85cfb09ac215f905029ab50990bb37ba3d70e9c2c0cefef4c5e156dc3722b32"
links: [automated-user-handoff, source-development, steering-preservation]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지

사용량 차단은 작업 지속·종료 정리보다 우선. 소진 확인 뒤 짧은 중단 안내만 허용, 체크포인트·지식 저장·검사·Git·정리·인계 도구는 새 요청의 정상 사용량 확인 뒤로 연기.
