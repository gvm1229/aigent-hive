---
schema_version: 1
pair_id: knowledge-storage
topic_slug: knowledge-storage
language: en
counterpart: ../ko/knowledge-storage.md
title: "Canonical Knowledge Storage"
summary: "Each selected Wiki backend is canonical and SQLite is always a disposable projection."
tags: [knowledge, sqlite]
aliases: ["Markdown SQLite boundary"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:9834a07f92cb41cb60c697f71aed30f8cc7874e338d51eff5a8a365a515a13e6"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:4b746d558c91b7cb0cacbef7c516b3cd1d1ddaacbd47c9c1f16bf33c4bff1ab4"
links: [docs-wiki-architecture, host-external-integrations, shared-index]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# Canonical Knowledge Storage

Source knowledge, run, role, and plan state remain tracked Markdown, YAML, or TOML.
Consumer Markdown mode keeps Wiki Markdown canonical; Notion mode keeps the selected
Notion scope canonical and creates no active local Wiki Markdown. SQLite stores only
a rebuildable local search projection and never owns the sole durable copy.
