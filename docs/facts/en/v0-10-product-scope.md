---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: en
counterpart: ../ko/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 Product Scope"
summary: "The 0.10.0 scope adds a Korean language core and reopens vector qualification with safe embedding, isolation, rollback, and conditional one-engine adoption."
tags: [knowledge, language, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:39231490f4083cba9cfaba64dbf265045ccd9cbcada90cd3646cdbd936932c19"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:d331dc879cf51eab078c5e189b2fe7b8d729e541"
status: active
---

# Aigent Hive 0.10.0 Product Scope

The scope includes relation search, lossless upgrade, host-neutral continuation, automatic Korean
language handling, and optional vector search. Non-vector repairs merged into develop and passed
test.4 acceptance on three operating systems. Vector functions are implemented and verified on a
dedicated branch. The maintainer accepted measured stress performance under revised limits on 2026-08-29.
Prior failures remain valid history. Vector-inclusive public test.6 passed installation acceptance on three operating systems. Stable integration,
publication, and installation require explicit version-specific approval.
