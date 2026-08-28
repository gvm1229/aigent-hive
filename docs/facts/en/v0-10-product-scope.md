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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:a00c93539ccde3105ec332ac6fcb40fdf9d43e580b702e1f7035cfc6cb36d088"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:d9cb7733237df4e5cc14824cb2df13ff75009776"
status: active
---

# Aigent Hive 0.10.0 Product Scope

The scope includes relation search, lossless upgrade, host-neutral continuation, automatic Korean
language handling, and optional vector search. Non-vector repairs merged into develop and passed
test.4 acceptance on three operating systems. Vector functions are implemented and verified on a
dedicated branch. The maintainer accepted measured stress performance under revised limits on 2026-08-29; public acceptance remains open.
Prior failures remain valid history. The next vector-inclusive public test is test.5. Stable integration,
publication, and installation require explicit version-specific approval.
