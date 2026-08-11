---
schema_version: 1
pair_id: knowledge-portability-scan
topic_slug: knowledge-portability-scan
language: ko
counterpart: ../en/knowledge-portability-scan.md
title: "Knowledge 이식과 directory scan"
summary: "v0.9의 checksummed canonical bundle, normalized collection, evidence-qualified scan과 automatic query 구현."
tags: [collection, knowledge, portability, scan, v0-9]
aliases: ["Directory knowledge scan", "Hive knowledge bundle"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:2dece311aef55de6a52b9f3f8f79fbf928009f312a98d7ab0c3cb09cfa9db741"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:976150863fbb552b17b456b5bdaf4f6ce2780dcd7ed9af45ebcf565aae709e05"
  - "repo:docs/research/knowledge-portability-ingestion-retrieval.md#sha256:983844189f92fca165ed1c85eadf975dc404b46ddea4111ab956823448b15de6"
links: [global-knowledge-rag, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# Knowledge 이식과 directory scan

Markdown mode v0.9 구현: canonical Markdown·portable metadata의 deterministic `.hivekb` 이식과
destination index rebuild, stable `collection_id`, evidence-qualified scan, reviewed root
promotion, bounded automatic query. SQLite의 portable source 사용 0건. Notion mode export는
이 구현 claim 밖. 100 collection·50,000 chunk qualification:
export p95 `1066.9209ms`, import+rebuild p95 `3255.1537ms`.
