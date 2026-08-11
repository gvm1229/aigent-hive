---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: ko
counterpart: ../en/v0-9-skill-suite-plan.md
title: "v0.9 Skill suite"
summary: "완료된 v0.9 Skill 기준선과 별도 default-off Hive-native 반복 실행 계획."
tags: [graph-engineering, rag, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop·Wiki 계획"]
sources:
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:c122052f10778e4c0e3c56c9511c2fdb6fc48528ba3d0dba599f91d3be77a5b5"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:2dece311aef55de6a52b9f3f8f79fbf928009f312a98d7ab0c3cb09cfa9db741"
  - "repo:docs/plans/active/native-iterative-execution.md#sha256:98c0ecbf5f659ea098520df1acec8af027e47fd0aa2fdf3c89de7751d9fd6d2a"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6763857d275d0a35065e27f744e4a7d2c83d77b876abcdb5343f37be01ffe35e"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:976150863fbb552b17b456b5bdaf4f6ce2780dcd7ed9af45ebcf565aae709e05"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:22df31e4312c84eeb17fdbe490a39223034b6a67e46f573148ea6132aea1e8e0"
  - "repo:docs/research/v0.9-omx-omc-capability-inventory.md#sha256:6ba67a8de8a2faf3e546de19403d3c35ec9815b8a4dc871ba06b1fe7511bc93d"
links: [docs-wiki-architecture, global-knowledge-rag, judge-verification, knowledge-portability-scan, orchestration-ownership, skill-routing]
reviewed_revision: "git:4e750ce659c953d7d71ab6e9536c29968ab1f028"
status: active
---

# v0.9 Skill suite

완료된 v0.9 기준선: graph engineering, canonical Markdown state, Wiki·RAG,
portable knowledge, cleanup, bounded research. Historical scheduler·Ralph·team
제외 정책: 별도 `NAT-*` 계획으로 superseded. Native iterative 실행: protocol·host
feasibility·qualification·동의·activation gate 전 default-off.
Strict iterative·team·multi-goal terminal gate는 호출 mode와 무관하게 authenticated Judge 필수.
Scheduler tick·retry별 Judge 호출 금지.
