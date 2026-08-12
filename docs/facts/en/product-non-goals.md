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
  - "repo:docs/overview/product.md#sha256:1418b29aac5b84acf17168c93d98cc0bb994d9895163dd94a78e4dad67b8bf29"
links: [orchestration-ownership, product-purpose]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# Product Non-goals

Hive is not a model runtime, provider session engine, provider API client,
credential store, or direct model/subagent process launcher. Hive-native logical
scheduling and iterative, team, and multi-goal control are permitted behind
implementation, host-qualification, consent, and activation gates.
