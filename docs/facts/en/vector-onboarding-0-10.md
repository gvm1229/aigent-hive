---
schema_version: 1
pair_id: vector-onboarding-0-10
topic_slug: vector-onboarding-0-10
language: en
counterpart: ../ko/vector-onboarding-0-10.md
title: "0.10.0 vector-search onboarding"
summary: "One saved vector-search answer, a short local claim, and a fixed-scope new-session setup prompt."
tags: [knowledge, onboarding, v0-10, vector]
aliases: ["Vector onboarding"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:2dfce7ec9ad595d35bc2da971a2f1578083b5679adaf58facf1295152777f66a"
  - "repo:docs/guides/vector-search.md#sha256:ec476f82aa26bba2e8a1605af7620974b4620ee33d1f855c0d7669fa10d5df18"
  - "repo:docs/plans/active/vector-onboarding-0.10.0.md#sha256:83ebcc975f8fbbbb09adebd8f28efe94a897face9c54a09de76cfb5971a9679a"
links: [hive-preserving-uninstall, hybrid-vector-search-0-10, v0-10-product-scope]
reviewed_revision: "git:64a9f1929b96fcd3a274f2dd0e86b7d9e7c4399c"
status: active
---

# `0.10.0` vector-search onboarding

User answer stored separately from setup and runtime state. `yes` creates a fixed-scope prompt;
`no` suppresses automatic prompting. A short local claim prevents concurrent duplicate questions
without storing a host session identifier. Update and preserving reinstall retain the answer.
Canonical Markdown, FTS, and existing derived vector material remain unchanged by the answer.
