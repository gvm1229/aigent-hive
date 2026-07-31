---
schema_version: 1
pair_id: orchestration-ownership
topic_slug: orchestration-ownership
language: en
counterpart: ../ko/orchestration-ownership.md
title: "Orchestration Ownership"
summary: "Compatible OMX or OMC owns orchestration; otherwise the host owns native support."
tags: [orchestration, ownership]
aliases: ["Orchestration owner"]
sources:
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:d180f7a9c22d525888e329e026a7b971e579f877c03dd9fee265967ab34cec69"
links: [product-non-goals, skill-routing, v0-9-skill-suite-plan]
reviewed_revision: "git:be5253bcbd0d9818333e5702d0ef9ce438ee4d62"
status: active
---

# Orchestration Ownership

Compatible OMX on Codex or OMC on Claude owns established orchestration. Otherwise
the active host owns only its truthful native capability. Hive does not switch a
pinned run owner silently. ADR-0015 proposes host-native Skill composition for v0.9
new runs while preserving existing owner pins.
