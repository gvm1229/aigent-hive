---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: en
counterpart: ../ko/host-external-integrations.md
title: "Discord v0.9 and Notion v0.10 Host Integrations"
summary: "Discord remains a v0.9 setup target; Notion connection and host OAuth are deferred to the first v0.10 test release."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:931d25f2b4d2673fbce477b5c0972bb54fe9b604b12f07ee541b92389dfcb6e8"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:2b819c1060972bb2416a751ff17e596094b00a6b"
status: active
---

# Discord v0.9 and Notion v0.10 Host Integrations

The Discord outbound notifier core and resumable setup primitive are implemented. Discord
webhook setup, project-aware alerts, and an HTML guide remain `DIS9-*` work. The typed
Notion backend, SQLite projection engine, and capability receipt validator are internal
v0.10 candidates; v0.9 setup, help, README, and release notes do not expose them.
Notion OAuth tokens, webhook URLs, raw prompts, and absolute paths remain excluded.
