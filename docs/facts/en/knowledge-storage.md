---
schema_version: 1
pair_id: knowledge-storage
topic_slug: knowledge-storage
language: en
counterpart: ../ko/knowledge-storage.md
title: "Canonical Knowledge Storage"
summary: "v0.9 keeps Markdown canonical and SQLite is always a disposable projection; Notion is deferred to v0.10 test."
tags: [knowledge, sqlite]
aliases: ["Markdown SQLite boundary"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:9834a07f92cb41cb60c697f71aed30f8cc7874e338d51eff5a8a365a515a13e6"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
links: [docs-wiki-architecture, host-external-integrations, shared-index]
reviewed_revision: "git:2b819c1060972bb2416a751ff17e596094b00a6b"
status: active
---

# Canonical Knowledge Storage

Source knowledge, run, role, and plan state remain tracked Markdown, YAML, or TOML.
The v0.9 consumer Wiki keeps local Markdown canonical. SQLite stores only a rebuildable
local search projection and never owns the sole durable copy. Notion canonical storage is
deferred until the first v0.10 test release.
