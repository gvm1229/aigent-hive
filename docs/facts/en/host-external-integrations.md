---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: en
counterpart: ../ko/host-external-integrations.md
title: "Discord and Notion Host Integrations"
summary: "The integration core exists, while end-to-end global setup, host OAuth handoff, and project-aware Discord alerts remain planned work."
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

# Discord and Notion Host Integrations

The typed Notion backend, SQLite projection engine, capability receipt validator,
and outbound Discord notifier are implemented. The user-facing connection layer is
not complete: global setup does not yet select and verify Notion through a host-owned
browser OAuth flow, configure and test Discord, or ship a visual HTML guide. The
`DNI-*` plan adds those paths and binds each guard alert to a safe project identity,
run, request summary, canonical progress, checkpoint, and resume hint. Raw prompts
remain excluded by default and require explicit opt-in, preview, and redaction.
