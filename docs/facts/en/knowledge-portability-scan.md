---
schema_version: 1
pair_id: knowledge-portability-scan
topic_slug: knowledge-portability-scan
language: en
counterpart: ../ko/knowledge-portability-scan.md
title: "Knowledge Portability and Directory Scan"
summary: "The v0.9 contract for checksummed canonical bundles, normalized collections, bulk scanning, and automatic queries."
tags: [collection, knowledge, portability, scan, v0-9]
aliases: ["Directory knowledge scan", "Hive knowledge bundle"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:ece47739f1d17b0d7ba604e5126fec55b445693335da10e54563b6cf2aa91224"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:437beec1bc0e37668162752ce8aa305ed73fc54a0ea27c5c7f3a4b160d9757f3"
  - "repo:docs/research/knowledge-portability-ingestion-retrieval.md#sha256:983844189f92fca165ed1c85eadf975dc404b46ddea4111ab956823448b15de6"
links: [global-knowledge-rag, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:4ef913efce07f4e86da98915c5ae5056dfac23e6"
status: active
---

# Knowledge Portability and Directory Scan

The v0.9 decision transfers canonical Markdown and portable metadata in a
checksummed `.hivekb` bundle, then rebuilds the destination index. Stable
`collection_id` rows replace per-directory tables. `hive-knowledge-scan`
separates evidence-qualified project claims from root promotion candidates,
while the existing `hive-knowledge-query` remains the bounded automatic owner.
