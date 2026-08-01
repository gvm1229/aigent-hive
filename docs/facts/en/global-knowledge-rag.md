---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: en
counterpart: ../ko/global-knowledge-rag.md
title: "Global Knowledge RAG"
summary: "v0.9 implements mandatory durable-memory writes and automatic retrieval for questions, research, and knowledge-dependent tasks."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:fb5917be58cbfad73a01a2c587b7773c6775d1bbd1f6aa3c8286a50b69999d3b"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:2b7b1132b276dc59c0a00076d8aca13aebcb75eefb2dd66a3e1f9d51494fbba9"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# Global Knowledge RAG

v0.9 expands `hive-knowledge-query` into the bounded automatic retrieval owner
and requires selected-backend canonical writes for reusable user facts,
preferences, and workflows. Markdown mode writes Markdown; Notion mode writes the
selected Notion scope and keeps SQLite derived. Named project and collection scope, citation-ready chunks,
fresh-session recall, and derived-only repair are implemented. The 50,000-chunk
qualification measured 163.3569ms cold p95 and 0.1178ms prepared-resident warm p95.
