---
schema_version: 1
pair_id: run-recovery
topic_slug: run-recovery
language: ko
counterpart: ../en/run-recovery.md
title: "Durable run recovery"
summary: "Canonical criterion·event·receipt·role state·evidence 기반 fresh-session resume."
tags: [recovery, run]
aliases: ["Fresh-session resume"]
sources:
  - "repo:docs/architecture/run-lifecycle.md#sha256:0f0c79a9eb97ec1901437b8f854d757445b262af045c7321dfca5cddf5a6c5a3"
links: [automatic-dispatch-guard, role-state]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# Durable run recovery

Fresh host session의 복구 source: canonical PLAN criterion, STATUS, event, typed
receipt, role handoff, evidence, control epoch. 현재 release 범위: model·subagent
spawn 없는 dispatch data 준비. Native 실행 계획: 동일 process 경계의 결정론적 복구.
