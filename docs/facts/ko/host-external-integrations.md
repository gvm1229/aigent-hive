---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord·Notion host integration"
summary: "Integration core 구현 완료, end-to-end global setup·host OAuth handoff·project-aware Discord 알림은 후속 계획."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9eed99de00f33af8c7b022efa62e28952ee7e516ef9e9f98fd0bd595d7e1577c"
  - "repo:docs/plans/active/discord-notion-onboarding.md#sha256:034aaea79a8cc792525ad1a5ea8b98c99bd4f22ce43f1045d06bb052b4d00e46"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:a8f2ef61565e15edef9e42355877f2d393058f80"
status: active
---

# Discord·Notion host integration

구현 완료 범위: typed Notion backend, SQLite projection engine, capability receipt 검증,
Discord outbound notifier core. 미완료 범위: global setup의 Notion 선택·host-owned browser
OAuth 연결 검증, Discord 설정·시험 알림, HTML 시각 안내. `DNI-*` 후속 범위:
안전한 project identity·run·요청 요약·canonical 진행 상태·checkpoint·재개 안내를
usage-guard 알림에 결합. 원문 prompt 기본 제외, 명시적 opt-in·preview·redaction 필수.
