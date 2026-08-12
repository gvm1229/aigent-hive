---
schema_version: 1
pair_id: automatic-dispatch-guard
topic_slug: automatic-dispatch-guard
language: ko
counterpart: ../en/automatic-dispatch-guard.md
title: "Automatic dispatch guard"
summary: "Unsafe automatic dispatch 차단용 preflight와 별도 authorization."
tags: [dispatch, guard, usage]
aliases: ["One-shot usage gate"]
sources:
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:5602025a4eb182cc6e51cc816cab74983f10ee2bdd2f6324649de63fdbddef1f"
links: [run-recovery, usage-sensor-policy]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Automatic dispatch guard

`hive usage enforce`: 새 automatic dispatch 직전 session-bound preflight. Exit success의
단독 dispatch 권한 없음. 별도 durable-run resume에서 exact brief 1개의 authorization
1개 발급·소비 필요.
