---
schema_version: 1
pair_id: adversarial-judge
topic_slug: adversarial-judge
language: en
counterpart: ../ko/adversarial-judge.md
title: "Adversarial Judge Skill"
summary: "0.10.0 adds an explicit adversarial-judge step that prepares a clean-context host-native Judge request and reuses authenticated Hive quorum verification."
tags: [judge, skills, v0-10]
aliases: ["Adversarial review"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:1645eb2249265b75d27b0c65a709806f4999a0ec425e8e874336bcda084b702c"
  - "repo:docs/decisions/product-release-decisions.md#sha256:25bd2880270b2dd21bf09d5efe576f4164b8d02fadd8366f8649d8d50d38bded"
  - "repo:docs/plans/active/adversarial-judge-0.10.0.md#sha256:c26efcfc99708c0c6edb6d1d3b4e0b473172a3c24c247938097e6515de7fdaf5"
links: [judge-verification, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:a2518fa364c40efb4e676fe31b694562f73dd819"
status: active
---

# Adversarial Judge Skill

Hive has package and quorum verification but no explicit Judge-launch step. `adversarial-judge`
prepares a clean-context request and a provider-neutral dispatch envelope. The active host launches
the separate Judge and returns typed evidence. Findings remain diagnostic until the existing
authenticated quorum contract authorizes acceptance. Hive never calls a provider or spawns a process.
