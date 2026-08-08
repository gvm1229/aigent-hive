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
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:bf987da6a220df4aa4194f87928626ea8321438671c9d4369c8e097fd272c8ec"
links: [docs-wiki-architecture, host-external-integrations, shared-index]
reviewed_revision: "git:907817827a6733dd380aaedea2e7592bc10a7311"
status: active
---

# Canonical Knowledge Storage

Source knowledge, run, role, and plan state remain tracked Markdown, YAML, or TOML.
Consumer Markdown mode keeps Wiki Markdown canonical; Notion mode keeps the selected
Notion scope canonical and creates no active local Wiki Markdown. SQLite stores only
a rebuildable local search projection and never owns the sole durable copy.
