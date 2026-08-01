---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord·Notion host integration"
summary: "Notion mode는 Notion 정본·SQLite 파생 상태, Discord는 guard outbound 알림부터 시작."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:4b746d558c91b7cb0cacbef7c516b3cd1d1ddaacbd47c9c1f16bf33c4bff1ab4"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:b97e5fdeff0be50747d147dad8f8b8c2dcc8487f0e54ea28decbc0da30cecf08"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# Discord·Notion host integration

Discord 초기 범위: usage guard outbound 알림. Claude 양방향 messaging:
공식 Channel plugin 위임. Codex 양방향 지원 조건: 공식 inbound session capability.
Wiki backend: user-scope `markdown|notion` 중 하나. Notion mode: selected Notion scope
유일 정본, active local Wiki Markdown 0건, user-root SQLite 삭제 가능 검색 projection.
매 user turn remote freshness 확인과 changed-only fetch 뒤 SQLite query. Notion 연결
우선순위: approved host plugin/app → official hosted MCP → explicit REST fallback.
