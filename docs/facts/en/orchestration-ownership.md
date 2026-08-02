---
schema_version: 1
pair_id: orchestration-ownership
topic_slug: orchestration-ownership
language: en
counterpart: ../ko/orchestration-ownership.md
title: "Orchestration Ownership"
summary: "Hive owns provider-neutral iterative control while hosts own model and subagent execution."
tags: [orchestration, ownership]
aliases: ["Orchestration owner"]
sources:
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:0400842448b5e73cedabe1d2eb941abf343a0e1564b2e161c8e54d6677af017e"
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:003a95d576041a8dfd3035b448a970919a2cb547c65a14035e8c789025113fa1"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:fc30a26c372c5dd0881e5fcc36742820f49ae62ef4ee8fac410a60e8d8509fc0"
links: [model-routed-custom-subagents, product-non-goals, skill-routing, v0-9-skill-suite-plan]
reviewed_revision: "git:150c4ddd84eb80b0906da731aceb81c4f2d8e059"
status: active
---

# Orchestration Ownership

ADR-0019 permits Hive-owned deterministic events, logical scheduling, leases,
receipts, cancellation, team coordination, and multi-goal state. The host retains
model and subagent execution. New workflows have no OMX/OMC dependency. Existing
external-owner runs remain read-only provenance; explicit migration creates a new
Hive-native run identity instead of switching an owner in place.
