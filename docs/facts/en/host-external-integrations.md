---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: en
counterpart: ../ko/host-external-integrations.md
title: "Discord and Notion Host Integrations"
summary: "Notion mode keeps Notion canonical and SQLite disposable while Discord starts with outbound guard alerts."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:4b746d558c91b7cb0cacbef7c516b3cd1d1ddaacbd47c9c1f16bf33c4bff1ab4"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:b97e5fdeff0be50747d147dad8f8b8c2dcc8487f0e54ea28decbc0da30cecf08"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# Discord and Notion Host Integrations

Discord starts with outbound usage-guard notifications. Claude delegates two-way
Discord messaging to the official Channel plugin; Codex support waits for an
official inbound session capability. The user selects one `markdown|notion` Wiki
backend. Notion mode keeps the selected Notion scope as its sole canonical source,
creates no active local Wiki Markdown, and uses user-root SQLite as a disposable
search projection. Each user turn checks remote freshness and fetches only changed
pages before querying SQLite. Access prefers an approved host plugin or app, then
Notion's hosted MCP, with explicitly consented REST as the final fallback.
