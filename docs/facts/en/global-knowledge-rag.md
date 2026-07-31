---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: en
counterpart: ../ko/global-knowledge-rag.md
title: "Global Knowledge RAG"
summary: "The accepted v0.9 plan adds mandatory durable-memory writes and fast user-root or cross-project retrieval before question routing."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:055496481dd5f0fa5ffcd92d6ddc6b456a01ce0db8edd998ccc3d2ae307f050e"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:4de98fc240cd60feb243a74ecbe4f46af79639f61d599a48f282cdc84b87ea3d"
links: [knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:6e3eb11fb43b99971f73e1fed471ea6b34e8ba33"
status: active
---

# Global Knowledge RAG

The accepted v0.9 plan upgrades the existing user-root FTS5 foundation into a
citation-ready RAG projection. Every question receives one bounded retrieval
preflight when the global Wiki is enabled. Reusable user-stated facts,
preferences, and workflows require an agent-reviewed canonical Markdown write,
including non-project preferences and explicit named-project scope. SQLite may
be replaced or extended for chunks, ranking, and incremental freshness, but it
remains disposable and never owns the sole copy of durable knowledge.
