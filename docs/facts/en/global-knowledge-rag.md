---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: en
counterpart: ../ko/global-knowledge-rag.md
title: "Global Knowledge RAG"
summary: "The accepted v0.9 plan adds mandatory durable-memory writes and automatic retrieval for questions, research, and knowledge-dependent tasks."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:ece47739f1d17b0d7ba604e5126fec55b445693335da10e54563b6cf2aa91224"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:0285b4850dfbb2651a2a8787c5d8c9b8c3b79cd3c3d1589c32106d2bb1847f43"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:4ef913efce07f4e86da98915c5ae5056dfac23e6"
status: active
---

# Global Knowledge RAG

The v0.9 plan expands the existing `hive-knowledge-query` into the bounded
automatic retrieval owner for questions, research, and knowledge-dependent
tasks. Reusable user facts, preferences, and workflows require canonical
Markdown writes with named project or collection scope. SQLite may change for
chunks, claims, ranking, and incremental freshness, but remains disposable.
