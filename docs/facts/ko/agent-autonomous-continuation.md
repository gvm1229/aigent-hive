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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d47f8ba8f5e3e88783cfc992cb88475d7cf47fa3459baecb434610e7f91c2d45"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:53d45f59e894a17eb3d804249d811b832f5bc1a47f7c3405ef897fd13fb37150"
  - "repo:crates/hive-render/src/lib.rs#sha256:51ce6acb616c554de9ae60e90259c815b30ddf33608ffdec02c07d966f2b089c"
  - "repo:harness/directives/00-project-harness.md#sha256:5d4e8b3c34a968b86735f59d449b194021c16ff8e9b4741b4c3ccaf130aa94b6"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:4c71dbbc592620844bff81765151014d4a657924d03aea602c08bc517e1c8727"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:b53ec86c9ad9e878ea0792b85145b9ba7db212242059a881b7c5c3218e1b14b9"
links: [automated-user-handoff, source-development, steering-preservation]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지

사용량 차단은 작업 지속·종료 정리보다 우선. 소진 확인 뒤 짧은 중단 안내만 허용, 체크포인트·지식 저장·검사·Git·정리·인계 도구는 새 요청의 정상 사용량 확인 뒤로 연기.
