---
schema_version: 1
pair_id: orchestration-ownership
topic_slug: orchestration-ownership
language: en
counterpart: ../ko/orchestration-ownership.md
title: "Orchestration Ownership"
summary: "Existing 0.8 runs retain their pinned owner; v0.9 new runs default to verified host-native capabilities."
tags: [orchestration, ownership]
aliases: ["Orchestration owner"]
sources:
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:f58554acf449855ca192ac1219d87019ca7ecc665506366455000bac78f24d87"
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:06938e887dc4992019718ea51ca0ec55f7bea4a56a647dd12409cd22c9375708"
links: [product-non-goals, skill-routing, v0-9-skill-suite-plan]
reviewed_revision: "git:d28c11908507cd0ae9f79ed0dfb4bcabf345ced2"
status: active
---

# Orchestration Ownership

ADR-0004 remains the historical contract for 0.8.x and existing runs, including
their pinned OMX, OMC, or host-native owner. Accepted ADR-0015 makes verified
host-native capability the default for v0.9 new runs. OMX and OMC become explicit,
user-selected compatibility layers rather than Hive dependencies. Hive never
switches a pinned run owner silently.
