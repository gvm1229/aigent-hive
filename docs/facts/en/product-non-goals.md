---
schema_version: 1
pair_id: product-non-goals
topic_slug: product-non-goals
language: en
counterpart: ../ko/product-non-goals.md
title: "Product Non-goals"
summary: "Hive does not own model execution, provider credentials, provider session engines, or direct process launch."
tags: [boundary, product]
aliases: ["Hive non-goals"]
sources:
  - "repo:docs/overview/product.md#sha256:fc6211acf48613b508f64b2a008eff0fc45a81ab7f3b07078fe3e030de48747f"
links: [orchestration-ownership, product-purpose]
reviewed_revision: "git:8a1e2b93fc5e1bf59ea54bdd7bfb4eb6bd63f6da"
status: active
---

# Product Non-goals

Hive is not a model runtime, provider session engine, provider API client,
credential store, or direct model/subagent process launcher. Hive-native logical
scheduling and iterative, team, and multi-goal control are permitted behind
implementation, host-qualification, consent, and activation gates.
