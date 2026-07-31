---
schema_version: 1
pair_id: knowledge-storage
topic_slug: knowledge-storage
language: en
counterpart: ../ko/knowledge-storage.md
title: "Canonical Knowledge Storage"
summary: "Tracked text is canonical and SQLite is a disposable projection."
tags: [knowledge, sqlite]
aliases: ["Markdown SQLite boundary"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:8bfd86a2ede49c3ce92f0a8e57a06c922c19248627d7d3552dd1777c1ee4954b"
links: [docs-wiki-architecture, shared-index]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Canonical Knowledge Storage

Durable knowledge lives in tracked Markdown, YAML, or TOML. SQLite stores only a
rebuildable local search projection and never owns the sole copy of a durable fact.
