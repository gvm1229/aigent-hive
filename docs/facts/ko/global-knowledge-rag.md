---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: ko
counterpart: ../en/global-knowledge-rag.md
title: "전역 knowledge RAG"
summary: "질문·research·knowledge task의 자동 검색과 durable memory 필수 기록을 추가한 v0.9 확정 계획."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:ece47739f1d17b0d7ba604e5126fec55b445693335da10e54563b6cf2aa91224"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:0285b4850dfbb2651a2a8787c5d8c9b8c3b79cd3c3d1589c32106d2bb1847f43"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:4ef913efce07f4e86da98915c5ae5056dfac23e6"
status: active
---

# 전역 knowledge RAG

v0.9 확정 범위: 기존 `hive-knowledge-query`를 질문·research·knowledge task의 bounded
automatic retrieval owner로 확장. Reusable user fact·preference·workflow의 canonical
Markdown write와 named project·collection scope 지원. SQLite의 chunk·claim·ranking·
incremental freshness schema 교체 허용, durable knowledge의 유일 정본 소유 금지.
