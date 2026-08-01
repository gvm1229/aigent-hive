---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: ko
counterpart: ../en/global-knowledge-rag.md
title: "전역 knowledge RAG"
summary: "질문·research·knowledge task의 자동 검색과 durable memory 필수 기록을 구현한 v0.9 결과."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:fb5917be58cbfad73a01a2c587b7773c6775d1bbd1f6aa3c8286a50b69999d3b"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:2b7b1132b276dc59c0a00076d8aca13aebcb75eefb2dd66a3e1f9d51494fbba9"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# 전역 knowledge RAG

v0.9 구현: `hive-knowledge-query`의 bounded automatic retrieval, reusable user
fact·preference·workflow의 selected-backend canonical 기록. Markdown mode는 Markdown,
Notion mode는 selected Notion scope 정본·SQLite 파생 상태. Named project·collection
scope, citation-ready chunk, fresh-session recall, derived-only repair. 50,000 chunk qualification:
cold p95 `163.3569ms`, prepared-resident warm p95 `0.1178ms`.
