---
schema_version: 1
pair_id: run-recovery
topic_slug: run-recovery
language: en
counterpart: ../ko/run-recovery.md
title: "Durable Run Recovery"
summary: "A fresh session resumes from canonical criteria, status, role, handoff, and evidence."
tags: [recovery, run]
aliases: ["Fresh-session resume"]
sources:
  - "repo:docs/architecture/run-lifecycle.md#sha256:24a50e2f5cdeeb465349d5fed86f9b6c83fe6dd9916456d078cbc499e7e2445a"
links: [automatic-dispatch-guard, role-state]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Durable Run Recovery

A fresh host session reconstructs continuation data from canonical PLAN criteria,
STATUS, role, handoff, evidence, and the immutable orchestration-owner pin. Hive
prepares dispatch data but does not spawn a model or subagent.
