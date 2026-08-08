---
schema_version: 1
pair_id: run-recovery
topic_slug: run-recovery
language: en
counterpart: ../ko/run-recovery.md
title: "Durable Run Recovery"
summary: "A fresh session resumes from canonical criteria, events, receipts, role state, and evidence."
tags: [recovery, run]
aliases: ["Fresh-session resume"]
sources:
  - "repo:docs/architecture/run-lifecycle.md#sha256:0f0c79a9eb97ec1901437b8f854d757445b262af045c7321dfca5cddf5a6c5a3"
links: [automatic-dispatch-guard, role-state]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# Durable Run Recovery

A fresh host session reconstructs work from canonical PLAN criteria, STATUS,
events, typed receipts, role handoff, evidence, and control epochs. Current
releases prepare dispatch data without spawning a model or subagent. Planned
native execution keeps that process boundary while adding deterministic recovery.
