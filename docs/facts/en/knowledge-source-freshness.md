---
schema_version: 1
pair_id: knowledge-source-freshness
topic_slug: knowledge-source-freshness
language: en
counterpart: ../ko/knowledge-source-freshness.md
title: "Historical Memory and Current Sources"
summary: "Canonical integrity and current source availability are separate checks."
tags: [freshness, knowledge, portability]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a72952ecc2423f82bd6710e9d88a88cd6b59f0d87500b30c1a1d68151879bcf1"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:aaea806d5543632cf6ebf6c9faee349e6d67a97f9b8c8bd753bd0da9e02582b7"
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:4298ffb4966ec809a35f01dc8276613d74544d8b2108109c8ae6732cb59183c4"
  - "repo:schemas/knowledge-retrieval-result.schema.json#sha256:6d8499cf211d663d928c3fa1a31a0e375e6c970736414a1298f1b42180953ac9"
links: [foundation-refactor, knowledge-portability-scan]
reviewed_revision: "git:1cdebf8aef9dde963f5aa88a36042186fbf37e80"
status: active
---

# Historical Memory and Current Sources

Canonical history remains readable when original source files are absent after transfer. Returned scan claims use `source_freshness`: `verified-current` confirms source bytes at that read; `historical-unverified` must not be used as current-code evidence. The CLI gives an explicit restore/rescan/review path. Canonical tampering, invalid origin bindings and observed source-content changes still fail closed. Missing evidence cannot hide another source mismatch. Retrieval never repairs canonical files or the index. This implements the maintainer’s 0.11.0 portability decision.
