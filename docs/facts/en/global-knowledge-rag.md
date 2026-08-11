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
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:24822777fdee6dec2272b659009913e69929aba5046d0858a9b745dec0e350c5"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:2b7b1132b276dc59c0a00076d8aca13aebcb75eefb2dd66a3e1f9d51494fbba9"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# Global Knowledge RAG

v0.9 expands `knowledge-recall` into the bounded automatic retrieval owner
and requires Markdown canonical writes for reusable user facts, preferences, and
workflows. Named project and collection scope, citation-ready chunks,
fresh-session recall, and derived-only repair are implemented. The 50,000-chunk
qualification measured 163.3569ms cold p95 and 0.1178ms prepared-resident warm p95.
Notion backend work is deferred to the first v0.10 test release.
