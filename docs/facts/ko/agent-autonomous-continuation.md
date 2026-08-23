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
  - "repo:.agents/directives/01-behavior.md#sha256:cf2be8b3eb6423c8cc3098ad96553544e37d8f13edc8c6cdc48262e39d82c662"
  - "repo:AGENTS.md#sha256:a541ce4a4aff8b3dcef37d728bef434073900bfed4cc45dc6c0384346b7308d1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:0360d325822c7b7407e12076c809b9c7a17fd189badcae469177ebd1acabed83"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a1c88321abacefb8eac2a6643d71421ee8f1283cce87bd3f41cdd201d18691c3"
  - "repo:crates/hive-render/src/lib.rs#sha256:421b619b2e7990bd2ceea9b2323231b60cb085f60fa1be8c0e62704f738ae7e4"
  - "repo:harness/directives/00-project-harness.md#sha256:8bced45c182a1fe5810cb5cd13e918e1b6dd50e7a24054bcf22a86d148243c8a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:c256f2dfdf227d99a56138d6099b0eaf99c12a8d4e49e831e45b553b5db142a9"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:ecbe2bbaa6391edc69f3490ca00dd14b9fc202e9ea03e7880939f6146ad97cbd"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:5257f45"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지
