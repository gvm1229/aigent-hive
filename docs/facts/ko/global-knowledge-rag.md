---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: ko
counterpart: ../en/global-knowledge-rag.md
title: "전역 knowledge RAG"
summary: "모든 질문 전 빠른 user-root·cross-project retrieval과 durable memory 필수 기록을 추가한 v0.9 확정 계획."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:055496481dd5f0fa5ffcd92d6ddc6b456a01ce0db8edd998ccc3d2ae307f050e"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:4de98fc240cd60feb243a74ecbe4f46af79639f61d599a48f282cdc84b87ea3d"
links: [knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:6e3eb11fb43b99971f73e1fed471ea6b34e8ba33"
status: active
---

# 전역 knowledge RAG

v0.9 확정 범위: 기존 user-root FTS5 기반의 citation-ready RAG projection 확대.
Global Wiki enabled 상태의 모든 질문 전 bounded retrieval 1회. 비project preference와
named-project scope를 포함한 reusable user fact·preference·workflow의 agent-reviewed
canonical Markdown write 필수. SQLite의 chunk·ranking·incremental freshness schema
교체·확장 허용, durable knowledge의 유일 정본 소유 금지.
