---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: ko
counterpart: ../en/global-knowledge-rag.md
title: "전역 knowledge RAG"
summary: "v0.9.0-test.16의 매 턴 자동 기록 배포·Windows 설치 완료. Codex 새 세션 수용과 정식 출시 검증 대기."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:196117cadc85737e0dbe35c8fcc6699e5180632d919782c2312453f588b3ab7a"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6049186f49dae584b981a8bb888ba15f43e7f61e085247f04b546ef368f7f6ce"
  - "repo:docs/plans/active/v0.9.0-knowledge-autocapture-regression.md#sha256:7fe35e6b1bdba462121104d9db09874ee755aa5c7ad65c85031353e35a172f0d"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:2234885542a2c3e82514121b890e129b89e5e563"
status: active
---

# 전역 knowledge RAG

정본 기록·검색·멱등성: 정상. `0.9.0-test.16`: 생성 English·Korean user guidance, Copier 투영,
plugin metadata·catalog의 범위 판정, 안전한 `remember` 1회, Markdown·index receipt 계약 배포.
Wiki 비활성 guidance: 기록 거부. Windows user install·설치 안내 검증 완료. Windows Codex 새 세션
기록·검색 수용과 replacement stable qualification: 출시 gate 유지.
