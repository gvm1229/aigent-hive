---
schema_version: 1
pair_id: knowledge-storage
topic_slug: knowledge-storage
language: en
counterpart: ../ko/knowledge-storage.md
title: "Canonical Knowledge Storage"
summary: "Markdown remains canonical; the Notion backend proposal was cancelled on 2026-10-05."
tags: [knowledge, sqlite]
aliases: ["Markdown SQLite boundary"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:9834a07f92cb41cb60c697f71aed30f8cc7874e338d51eff5a8a365a515a13e6"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:160bc8bc434f1547e1fb3dad23902b740304323c800b92ece62ccda10a61114e"
links: [docs-wiki-architecture, host-external-integrations, shared-index]
reviewed_revision: "git:e91331b5a498484c8abde81fe0989df1bccf8ea6"
status: active
---

# Canonical Knowledge Storage

Source knowledge, run, role, and plan state use tracked Markdown, YAML, or TOML.
Consumer knowledge uses local Markdown. SQLite is a rebuildable search index and must never
hold the only durable copy. The maintainer cancelled the Notion canonical backend proposal
on 2026-10-05. The old v0.10 target is no longer a planned delivery. Existing Discord behavior
is unaffected by that decision.
