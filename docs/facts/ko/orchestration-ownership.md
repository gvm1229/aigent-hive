---
schema_version: 1
pair_id: orchestration-ownership
topic_slug: orchestration-ownership
language: ko
counterpart: ../en/orchestration-ownership.md
title: "Orchestration ownership"
summary: "Hive의 provider-neutral 반복 제어 소유, host의 model·subagent 실행 소유."
tags: [orchestration, ownership]
aliases: ["Orchestration owner"]
sources:
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:0400842448b5e73cedabe1d2eb941abf343a0e1564b2e161c8e54d6677af017e"
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:c122052f10778e4c0e3c56c9511c2fdb6fc48528ba3d0dba599f91d3be77a5b5"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:a15a00e40b63abb6aa312ed24ee3d80c491f0b79056fa628e165431858e51551"
links: [judge-verification, model-routed-custom-subagents, product-non-goals, skill-routing, v0-9-skill-suite-plan]
reviewed_revision: "git:ffdfb476d4e21dafe5d4dc896fa272f7244d0fe1"
status: active
---

# Orchestration ownership

ADR-0019의 Hive 소유 범위: deterministic event, logical scheduler, lease, receipt,
cancel, team coordination, multi-goal state. Host 소유 범위: model·subagent 실행.
신규 workflow의 OMX·OMC dependency 없음. 기존 external-owner run: read-only
provenance. 명시적 migration: in-place owner switch 대신 새 Hive-native run identity.
Strict iterative·team·multi-goal terminal gate는 호출 mode와 무관하게 authenticated Judge 필수.
Tick·retry별 Judge 호출 금지.
