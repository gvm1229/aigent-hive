---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: en
counterpart: ../ko/host-external-integrations.md
title: "Discord and Notion Host Integrations"
summary: "Host-native integrations come first while Markdown and SQLite retain Hive knowledge authority."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:1ad38b670e5b9f05d495efeea623a1e5fa66a97288ef63a08078c26e789d369e"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:507cdf98de2b0873b0e554fd1bc53810b11c7dc0"
status: active
---

# Discord and Notion Host Integrations

Discord starts with outbound usage-guard notifications. Claude delegates two-way
Discord messaging to the official Channel plugin; Codex support waits for an
official inbound session capability. Notion access prefers each host's approved
plugin or app, then Notion's hosted MCP, with REST as the final fallback. Reviewed
Notion content must still materialize into project Markdown before rebuilding the
derived SQLite index.
