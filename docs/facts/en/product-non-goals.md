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
  - "repo:docs/overview/product.md#sha256:3c0ca860f0286357626c89b0052c68668cd8deccb596ca3cf9f4b1e9ab45edb4"
links: [orchestration-ownership, product-purpose]
reviewed_revision: "git:c5b13bc26c4b2485dcad7509bc8590f97d0831fe"
status: active
---

# Product Non-goals

Hive is not a model runtime, provider session engine, provider API client,
credential store, or direct model/subagent process launcher. Hive-native logical
scheduling and iterative, team, and multi-goal control are permitted behind
implementation, host-qualification, consent, and activation gates.
