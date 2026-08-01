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
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:ece47739f1d17b0d7ba604e5126fec55b445693335da10e54563b6cf2aa91224"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:2b7b1132b276dc59c0a00076d8aca13aebcb75eefb2dd66a3e1f9d51494fbba9"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:d28c11908507cd0ae9f79ed0dfb4bcabf345ced2"
status: active
---

# 전역 knowledge RAG

v0.9 구현: `hive-knowledge-query`의 bounded automatic retrieval, reusable user
fact·preference·workflow의 canonical Markdown 필수 기록, named project·collection scope,
citation-ready chunk, fresh-session recall, derived-only repair. 50,000 chunk qualification:
cold p95 `168.6215ms`, prepared-resident warm p95 `0.1367ms`.
