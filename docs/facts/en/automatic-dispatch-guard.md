---
schema_version: 1
pair_id: automatic-dispatch-guard
topic_slug: automatic-dispatch-guard
language: en
counterpart: ../ko/automatic-dispatch-guard.md
title: "Automatic Dispatch Guard"
summary: "A usage preflight blocks unsafe automatic dispatch but never authorizes it alone."
tags: [dispatch, guard, usage]
aliases: ["One-shot usage gate"]
sources:
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:141e8070b475ee2b0d81e93a69217093e07af9a9ca61c16dcbb31f111ea1d0f4"
links: [run-recovery, usage-sensor-policy]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Automatic Dispatch Guard

`hive usage enforce` is a session-bound preflight immediately before a new automatic
dispatch. Exit success does not authorize dispatch; a separate durable-run resume must
issue and consume one exact authorization for one brief.
