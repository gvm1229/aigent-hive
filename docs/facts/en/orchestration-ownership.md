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
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:0888b22473297bf6161141b508e7e276d5c8cc3bf5ffe9c43269b16c3fec347e"
links: [product-non-goals, skill-routing]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Orchestration Ownership

Compatible OMX on Codex or OMC on Claude owns established orchestration. Otherwise
the active host owns only its truthful native capability. Hive does not switch a
pinned run owner silently.
