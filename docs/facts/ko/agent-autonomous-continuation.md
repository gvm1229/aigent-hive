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
  - "repo:.agents/directives/01-behavior.md#sha256:42bbd59e702cdce48ac6396d4c5a2f3a9b7574cd99272e22f3279c00b041cba4"
  - "repo:.agents/directives/04-documentation-state.md#sha256:44913afc655f527245720594f16a92c87061abfc28280f1a2834ad328b336be5"
  - "repo:.agents/directives/06-session-coordination.md#sha256:a24536201b77619549620d88612c186b769e90a774043895370a064779d8d758"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:crates/hive-render/src/lib.rs#sha256:019c4b9187834d210c659a1ade13f9a30d5b04c45088e5184e04d0340797712e"
  - "repo:docs/plans/active/agent-autonomous-continuation.md#sha256:83b8604202102dc5424c63648833d99978d2d415b974e22c7d70fc511f1c5883"
  - "repo:harness/directives/00-project-harness.md#sha256:fb6cb8107a38aa3fe70040d4e730e53190a66ed6047a8e40f55acf811425d87d"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
  - "repo:tests/conformance/test_phase3_static_contracts.py#sha256:817159015348378befeca4653a6c2bc4f8b9c4eb06a2504975e33a253955dbfa"
  - "repo:tests/fixtures/agent-autonomous-continuation.json#sha256:168eb72b79508187e841e5caf25d88f15a86a43b10f0327d7c1ce5a8226aa934"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:9a125333ed070140b3773462d895684cba62fe6b"
status: active
---

# Agent 자율 실행 지속

source·소비자 Agent 공통 범위: 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업.
소비자 프로젝트 생성 지침과 영어·한국어 전역 설정에도 같은 계속 실행 계약 적용.
최종 종료 조건: Agent 소유 작업 `0건`. 사용자 권한·외부 증거·차단 상태: 정확한 owner·증거·복구 경로 기록.
