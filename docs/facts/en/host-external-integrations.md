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
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:bf987da6a220df4aa4194f87928626ea8321438671c9d4369c8e097fd272c8ec"
  - "repo:docs/plans/active/discord-notion-onboarding.md#sha256:9a444a6787fa527ab5ea96e09bef31610575e700676bd6562739f3f59f5b2a11"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:907817827a6733dd380aaedea2e7592bc10a7311"
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
