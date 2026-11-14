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
  - "repo:AGENTS.md#sha256:0588c67a687a50d00387ab18e2dda2d79a8bcf3344b67f63849cd5f8cab0dabb"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9903f35f13ae28c9853cdf99ab17c42876e8a5b12c91100e7f4c51cc04147c09"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:34e8f7c0d827e90ed98a1b8832f8a2ab4b19f736ea67135f6fe71f728bf87287"
  - "repo:crates/hive-render/src/lib.rs#sha256:0ab91f6389a28a312d90f0aa6ccd5b783da55b2bc21227d533448b6426e548d7"
  - "repo:harness/directives/00-project-harness.md#sha256:7073d74da76915240b5061818f9a00644872a3b5ff7d84fa55e30dc26c99a50c"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:4c71dbbc592620844bff81765151014d4a657924d03aea602c08bc517e1c8727"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:edf491a6826fe3c171e84b8ec3b25bc41d5be32a397840fd91cbdbd74a947985"
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
