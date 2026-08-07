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
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:4d6b1e5a018e5ef5ed129323927c191c1d74208a8c3d2d5b05678096629e9f82"
  - "repo:docs/plans/active/discord-notion-onboarding.md#sha256:acdd99039a9b030af94549ba7fb7eb9c9fbf6d51002ba0584082f0d623a3c6dc"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:b97e5fdeff0be50747d147dad8f8b8c2dcc8487f0e54ea28decbc0da30cecf08"
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
