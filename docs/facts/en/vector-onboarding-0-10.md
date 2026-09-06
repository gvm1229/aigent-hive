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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/guides/vector-search.md#sha256:ec476f82aa26bba2e8a1605af7620974b4620ee33d1f855c0d7669fa10d5df18"
  - "repo:docs/plans/active/vector-onboarding-0.10.0.md#sha256:96f2f93129dc8ac8d7a70789e940b57d217d360ae112ea1c91521097abf2b086"
links: [hive-preserving-uninstall, hybrid-vector-search-0-10, v0-10-product-scope]
reviewed_revision: "git:64a9f1929b96fcd3a274f2dd0e86b7d9e7c4399c"
status: active
---

# `0.10.0` vector-search onboarding

User answer stored separately from setup and runtime state. `yes` creates a fixed-scope prompt;
`no` suppresses automatic prompting. A short local claim prevents concurrent duplicate questions
without storing a host session identifier. Update and preserving reinstall retain the answer.
Canonical Markdown, FTS, and existing derived vector material remain unchanged by the answer.
