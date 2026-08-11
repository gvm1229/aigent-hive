---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: ko
counterpart: ../en/global-knowledge-rag.md
title: "전역 knowledge RAG"
summary: "v0.9 operational guidance의 매 턴 자동 기록 보정 완료. Windows Codex 새 세션 수용과 출시 검증 대기."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:2dece311aef55de6a52b9f3f8f79fbf928009f312a98d7ab0c3cb09cfa9db741"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6763857d275d0a35065e27f744e4a7d2c83d77b876abcdb5343f37be01ffe35e"
  - "repo:docs/plans/active/v0.9.0-knowledge-autocapture-regression.md#sha256:1e814cdd5a4f4f0806e2dab7789d7dd2ffd4df86d55256cbe292880e3d44e7b7"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:2234885542a2c3e82514121b890e129b89e5e563"
status: active
---

# 전역 knowledge RAG

정본 기록·검색·멱등성: 정상. 생성 English·Korean user guidance: 범위 판정, 안전한 `remember`
1회, Markdown·index receipt 필수. Wiki 비활성 guidance: 기록 거부. 세 host 투영·localized
Skill metadata: 자동 경로 보존. Windows Codex 새 세션 기록·검색 수용과 replacement stable
qualification: 출시 gate 유지.
