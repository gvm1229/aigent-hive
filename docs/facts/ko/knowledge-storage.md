---
schema_version: 1
pair_id: knowledge-storage
topic_slug: knowledge-storage
language: ko
counterpart: ../en/knowledge-storage.md
title: "Canonical knowledge 저장"
summary: "Tracked text 정본과 disposable SQLite projection."
tags: [knowledge, sqlite]
aliases: ["Markdown SQLite 경계"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:8bfd86a2ede49c3ce92f0a8e57a06c922c19248627d7d3552dd1777c1ee4954b"
links: [docs-wiki-architecture, shared-index]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Canonical knowledge 저장

Durable knowledge 정본: tracked Markdown·YAML·TOML. SQLite 역할: 재생성 가능한
local search projection. Durable fact의 유일한 저장소로 사용 금지.
