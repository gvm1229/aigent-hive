---
schema_version: 1
pair_id: agent-autonomous-continuation
topic_slug: agent-autonomous-continuation
language: ko
counterpart: ../en/agent-autonomous-continuation.md
title: "Agent 자율 실행 지속"
summary: "Agent 소유 작업 잔존 상태: 진행 보고 종료 금지"
tags: [agent, completion, regression]
aliases: ["중간 종료 방지"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:37e56019efc3b863734395bde48de0c2e1a3abf4aab8b2be982533fcb2ef6097"
  - "repo:.agents/directives/04-documentation-state.md#sha256:2b1909a619ca2b270dd049df9ad91f892f6fd2734e97e6869c421fe9c5a75090"
  - "repo:.agents/directives/06-session-coordination.md#sha256:884fedad85a6bd5c7865b5fc6be9b132c4653abb8d685f26aff621596f6ae48a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:8a2fc46f070b2d093b5826cf59b19a718a6ffbcb71d1516212f1b9a584f42c50"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0bfd9117a0d835da5f19bc02b82959a5630a4955a81ee3efda0a6ba5246dfaad"
  - "repo:crates/hive-render/src/lib.rs#sha256:2432d0bf172063791159b60582dbfd3fdfcd63bbd7e1cb1cf13c03092e6c2104"
  - "repo:docs/plans/active/agent-autonomous-continuation.md#sha256:83b8604202102dc5424c63648833d99978d2d415b974e22c7d70fc511f1c5883"
  - "repo:harness/directives/00-project-harness.md#sha256:da0203d47899f2e045560b3ac718c9f22775ab6edf638315e9b2e535ac27e9b4"
  - "repo:harness/template/AGENTS.md.jinja#sha256:d706dc6585c1bbaa820d328ebfaae919cd02496adac0acec373ee4d0e37afe56"
  - "repo:tests/conformance/test_phase3_static_contracts.py#sha256:a84c26b9c008b89b7131bc63401a9801ebee1f938873fb03f94ed071be499e35"
  - "repo:tests/fixtures/agent-autonomous-continuation.json#sha256:168eb72b79508187e841e5caf25d88f15a86a43b10f0327d7c1ce5a8226aa934"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:b8e8ea1d68fc2b37a37b07d5d287c3b40c48edf8"
status: active
---

# Agent 자율 실행 지속

source·소비자 Agent 공통 범위: 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업.
소비자 프로젝트 생성 지침과 영어·한국어 전역 설정에도 같은 계속 실행 계약 적용.
최종 종료 조건: Agent 소유 작업 `0건`. 사용자 권한·외부 증거·차단 상태: 정확한 owner·증거·복구 경로 기록.
