---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: en
counterpart: ../ko/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 Product Scope"
summary: "0.10.0 is the stable release of the Korean language core, optional vector search, relationship search, lossless knowledge transfer, and verified continuation features."
tags: [knowledge, language, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:2b8007e0cbf5a0f89ebb654ee7f6b44a1b203eee905205fe7ea90629941e4cad"
  - "repo:docs/public-stable-release.json#sha256:3828fade92ec45cdc0eab834aaf8029d95f2619ebc87e034172898371e65668e"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:301147fab8252954b29b7393327dfcff18eb8b1d"
status: active
---

# Aigent Hive 0.10.0 Product Scope

`0.10.0` is stable as of 2026-09-02. It includes relationship search, lossless knowledge transfer,
host-neutral verified continuation, automatic Korean language handling, and optional vector search.
The public test passed installation acceptance on Windows x64, macOS arm64, and Linux musl; the
stable candidate and publication then passed from the approved main source. The vector-search
stress policy remains documented with its limits, and SQLite full-text search remains available.
