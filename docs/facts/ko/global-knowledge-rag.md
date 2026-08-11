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
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:24822777fdee6dec2272b659009913e69929aba5046d0858a9b745dec0e350c5"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:dbeb3ee4cd8fbc2ca363c2d32aaa092e7dc1d0f884851925811ed1d580b48f1f"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:2234885542a2c3e82514121b890e129b89e5e563"
status: active
---

# 전역 knowledge RAG

정본 `hive knowledge remember` 기록·파생 검색·동일 입력 무변경: 정상. `0.9.0-test.13`
Windows Codex 설치본의 operational user guidance: every-turn 판정·명령·receipt 규칙 누락.
Localized `knowledge-capture` 설명: mandatory route 의미 소실. Project guidance에는 규칙 보유.
결과: 미등록 project와 일반 global turn의 reusable user fact 누락 가능. 기존 validate:
expected byte 무결성 증명, semantic gate 존재 여부 미증명.
