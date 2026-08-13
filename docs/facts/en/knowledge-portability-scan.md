---
schema_version: 1
pair_id: knowledge-portability-scan
topic_slug: knowledge-portability-scan
language: en
counterpart: ../ko/knowledge-portability-scan.md
title: "Knowledge Portability and Directory Scan"
summary: "v0.9 implements checksummed canonical bundles, normalized collections, evidence-qualified scanning, automatic queries, and 50,000-chunk qualification."
tags: [collection, knowledge, portability, scan, v0-9]
aliases: ["Directory knowledge scan", "Hive knowledge bundle"]
sources:
  - "repo:crates/hive-wiki/tests/v09_bundle_qualification.rs#sha256:503ae836a837ec541a6217d8afbc9160c9031b21320409e85e961ff8ed1d7005"
  - "repo:crates/hive-wiki/tests/v09_rag_qualification.rs#sha256:5ab69e6461b2583aa2bdc095ffe71b05476cde24e92d9d76ba81bf0b40f68a4c"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:196117cadc85737e0dbe35c8fcc6699e5180632d919782c2312453f588b3ab7a"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:976150863fbb552b17b456b5bdaf4f6ce2780dcd7ed9af45ebcf565aae709e05"
  - "repo:docs/research/knowledge-portability-ingestion-retrieval.md#sha256:983844189f92fca165ed1c85eadf975dc404b46ddea4111ab956823448b15de6"
links: [global-knowledge-rag, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:159e11c2f37f760b3e2bafedfb8b74cc735ff5f4"
status: active
---

# Knowledge Portability and Directory Scan

Markdown mode v0.9 supports deterministic `.hivekb` bundles, normalized
`collection_id` rows, evidence-qualified scans, root promotion, and bounded automatic
queries. SQLite is not portable source material; Notion-mode export is outside this claim.

Qualification: 25 deliberately long Wiki pages × 2,000 retrieval-sized chunks =
50,000 chunks, plus a registry of 100 portable collections. On 2026-08-13, release-build
local-SSD p95 results passed: fresh/warm retrieval `169.7758 ms`/`0.1343 ms` against
`500 ms`/`100 ms`; bundle export/import-rebuild `1042.8145 ms`/`3267.0567 ms` against
`5 s`/`15 s`. A chunk is not a whole document; results are qualification evidence, not a
hardware-independent guarantee.
