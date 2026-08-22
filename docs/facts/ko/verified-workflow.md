---
schema_version: 1
pair_id: verified-workflow
topic_slug: verified-workflow
language: ko
counterpart: ../en/verified-workflow.md
title: "Verified workflow Skill"
summary: "ralph-loop를 verified-workflow로 rename하고 복잡한 자연어 continuation을 evidence-gated 실행 graph로 routing하는 0.10.0 결정"
tags: [orchestration, skills, v0-10]
aliases: ["ralph-loop"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:c9399a4ad9e99389eb84aa5da5dbbae38cf20d5bca2f6c239322f2c81ad48d71"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:26e5fd299f961d79c6b8237c212b4b07e9e99770"
status: active
---

# Verified workflow Skill

- Public identity: `ralph-loop` → `verified-workflow`
- Protocol merge: `iterative-execution` receipt·dispatch uncertainty·budget·cancel·recovery 흡수
- 자동 route: Dependency·중간 evidence·bounded retry·독립 검증·steering·exact recovery signal 2개 이상
- 자동 route 금지: 긴 작업 또는 bare `continue`만 존재
- 실행 owner: Active host
