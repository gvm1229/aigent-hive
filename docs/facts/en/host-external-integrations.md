---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: en
counterpart: ../ko/host-external-integrations.md
title: "Discord and Notion Host Integrations"
summary: "The integration core exists, while resumable end-to-end global setup, host OAuth handoff, and project-aware Discord alerts remain planned work."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:2e268d2a33c699c6b77a5c711df6a50eaf95624964dc616848bf29321de3624d"
  - "repo:docs/plans/active/discord-notion-onboarding.md#sha256:0b5eb5f5b735c6c29cefe5c6fa1d034f32ffd6d4c6bc4ea45bcde86ed0e43702"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:2b819c1060972bb2416a751ff17e596094b00a6b"
status: active
---

# Discord and Notion Host Integrations

The typed Notion backend, SQLite projection engine, capability receipt validator, and
outbound Discord notifier are implemented. The user-facing connection layer is not
complete: global setup does not yet guide and verify Notion browser OAuth, configure
and test a Discord webhook, or ship a visual HTML guide. The `DNI-*` plan requires a
non-secret setup checkpoint after a failed or ended connection step. On the next
configuration run, the user can review everything, review selected settings, or
continue from that step after Hive rechecks saved answers and the connection receipt.
Notion OAuth tokens, webhook URLs, raw prompts, and absolute paths remain excluded.
