---
schema_version: 1
pair_id: v0-9-skill-suite-plan
topic_slug: v0-9-skill-suite-plan
language: en
counterpart: ../ko/v0-9-skill-suite-plan.md
title: "v0.9 Skill Suite"
summary: "The completed v0.9 Skill baseline now feeds a separate default-off Hive-native iterative execution plan."
tags: [graph-engineering, rag, skill-suite, v0-9, wiki]
aliases: ["v0.9 loop and Wiki plan"]
sources:
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:003a95d576041a8dfd3035b448a970919a2cb547c65a14035e8c789025113fa1"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:fb5917be58cbfad73a01a2c587b7773c6775d1bbd1f6aa3c8286a50b69999d3b"
  - "repo:docs/plans/active/native-iterative-execution.md#sha256:9922652d595d07189362bce90546b7e5077d53f6a8c842570240acf8ee27ba31"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:2b7b1132b276dc59c0a00076d8aca13aebcb75eefb2dd66a3e1f9d51494fbba9"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:976150863fbb552b17b456b5bdaf4f6ce2780dcd7ed9af45ebcf565aae709e05"
  - "repo:docs/plans/active/v0.9.0-loop-wiki-skills.md#sha256:22df31e4312c84eeb17fdbe490a39223034b6a67e46f573148ea6132aea1e8e0"
  - "repo:docs/research/v0.9-omx-omc-capability-inventory.md#sha256:4576fd514c873efcb016c86f2f3c4958216990b02480c3d04e756a52872d44bc"
links: [docs-wiki-architecture, global-knowledge-rag, knowledge-portability-scan, orchestration-ownership, skill-routing]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# v0.9 Skill Suite

The completed v0.9 baseline provides graph engineering, canonical Markdown state,
Wiki/RAG, portable knowledge, cleanup, and bounded research. Its historical
scheduler, Ralph, and team exclusions are superseded by the separate `NAT-*`
plan. Native iterative execution remains default-off until protocol, host
feasibility, qualification, consent, and activation gates pass.
