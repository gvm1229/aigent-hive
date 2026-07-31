---
schema_version: 1
pair_id: knowledge-portability-scan
topic_slug: knowledge-portability-scan
language: ko
counterpart: ../en/knowledge-portability-scan.md
title: "Knowledge 이식과 directory scan"
summary: "v0.9의 checksummed canonical bundle, normalized collection, bulk scan과 automatic query 계약."
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

# Knowledge 이식과 directory scan

v0.9 결정: SQLite file 대신 canonical Markdown·portable metadata를 담은 checksummed
`.hivekb` export·import와 destination index rebuild. Directory별 table 대신 stable
`collection_id` row 사용. `hive-knowledge-scan`은 evidence-qualified project claim과
root promotion candidate를 분리하고, 검색은 기존 `hive-knowledge-query`가 bounded
automatic owner로 유지.
