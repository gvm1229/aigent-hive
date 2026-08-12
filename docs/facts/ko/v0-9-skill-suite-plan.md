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
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:196117cadc85737e0dbe35c8fcc6699e5180632d919782c2312453f588b3ab7a"
  - "repo:docs/plans/active/native-iterative-execution.md#sha256:1e47ee71fca85dc30108bb5348acd2da3548b779bc786805e3cab124bd51ed16"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6049186f49dae584b981a8bb888ba15f43e7f61e085247f04b546ef368f7f6ce"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:976150863fbb552b17b456b5bdaf4f6ce2780dcd7ed9af45ebcf565aae709e05"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:22df31e4312c84eeb17fdbe490a39223034b6a67e46f573148ea6132aea1e8e0"
  - "repo:docs/research/v0.9-omx-omc-capability-inventory.md#sha256:cde76e53aa9e9921e26b2c15f4dc975d75e55075fad2c482f23d080abe6005c8"
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
