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
  - "repo:docs/archive/plans/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
  - "repo:docs/archive/plans/release-0.10.0.md#sha256:2b8007e0cbf5a0f89ebb654ee7f6b44a1b203eee905205fe7ea90629941e4cad"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:71a8c4e0f75fdc3f99aa7f4d1c4e2ace759af640dbc4dd013174639e71a3f01b"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:docs/public-stable-release.json#sha256:f212c4bf4c1c81e049c0103da39e69f0ac52f7b1a76ae2001d4af078f8de3992"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:c5b13bc26c4b2485dcad7509bc8590f97d0831fe"
status: active
---

# Aigent Hive 0.10.0 Product Scope

`0.10.0` is stable as of 2026-09-02. It includes relationship search, lossless knowledge transfer,
host-neutral verified continuation, automatic Korean language handling, and optional vector search.
The public test passed installation acceptance on Windows x64, macOS arm64, and Linux musl; the
stable candidate and publication then passed from the approved main source. The vector-search
stress policy remains documented with its limits, and SQLite full-text search remains available.
