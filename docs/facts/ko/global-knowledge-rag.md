---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: ko
counterpart: ../en/global-knowledge-rag.md
title: "전역 knowledge RAG"
summary: "v0.9 durable-memory 저장은 동작하지만 operational user guidance의 every-turn 필수 기록 gate 누락."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:2dece311aef55de6a52b9f3f8f79fbf928009f312a98d7ab0c3cb09cfa9db741"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6763857d275d0a35065e27f744e4a7d2c83d77b876abcdb5343f37be01ffe35e"
  - "repo:docs/plans/active/v0.9.0-knowledge-autocapture-regression.md#sha256:5cddc7bd9f4ab4c1b868d8a6a86cf155503d008ff901276158e64afc447c841a"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:2234885542a2c3e82514121b890e129b89e5e563"
status: active
---

# 전역 knowledge RAG

정본 기록·검색·멱등성: 정상. `0.9.0-test.13` Windows Codex user guidance: every-turn
명령·receipt 누락. Localized Skill metadata: mandatory route 의미 소실. 결과: 미등록
repository의 reusable fact 누락 가능. 기존 validate는 byte 일치만 증명. `KAC-001–008`:
guidance·routing·semantic test와 수동 호출 없는 Windows 새 세션 기록·검색이 통과할 때까지
stable 게시 차단.
