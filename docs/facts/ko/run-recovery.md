---
schema_version: 1
pair_id: run-recovery
topic_slug: run-recovery
language: ko
counterpart: ../en/run-recovery.md
title: "Durable run recovery"
summary: "Canonical criterion·status·role·handoff·evidence 기반 fresh-session resume."
tags: [recovery, run]
aliases: ["Fresh-session resume"]
sources:
  - "repo:docs/architecture/run-lifecycle.md#sha256:488841374212363c27c88e2358176f231c402ab365645d7e43d588eca749e742"
links: [automatic-dispatch-guard, role-state]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Durable run recovery

Fresh host session의 continuation source: canonical PLAN criterion, STATUS, role,
handoff, evidence, immutable orchestration-owner pin. Hive 범위: dispatch data 준비.
Model·subagent spawn 제외.
