---
schema_version: 1
pair_id: knowledge-portability-scan
topic_slug: knowledge-portability-scan
language: en
counterpart: ../ko/knowledge-portability-scan.md
title: "Knowledge Portability and Directory Scan"
summary: "v0.9 implements checksummed canonical bundles, normalized collections, evidence-qualified scanning, and automatic queries."
tags: [collection, knowledge, portability, scan, v0-9]
aliases: ["Directory knowledge scan", "Hive knowledge bundle"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:24822777fdee6dec2272b659009913e69929aba5046d0858a9b745dec0e350c5"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:976150863fbb552b17b456b5bdaf4f6ce2780dcd7ed9af45ebcf565aae709e05"
  - "repo:docs/research/knowledge-portability-ingestion-retrieval.md#sha256:983844189f92fca165ed1c85eadf975dc404b46ddea4111ab956823448b15de6"
links: [global-knowledge-rag, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# Knowledge Portability and Directory Scan

In Markdown mode, v0.9 transfers canonical Markdown and portable metadata through deterministic
`.hivekb` bundles and rebuilds the destination index. Stable `collection_id`
rows replace per-directory tables. Evidence-qualified scanning, reviewed root
promotion, and bounded automatic queries are implemented. SQLite is never the
portable source; Notion-mode export is outside this implemented claim. The 100-collection,
50,000-chunk qualification measured 1066.9209ms export p95 and 3255.1537ms
import-plus-rebuild p95.
