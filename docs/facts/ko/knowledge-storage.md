---
schema_version: 1
pair_id: knowledge-storage
topic_slug: knowledge-storage
language: ko
counterpart: ../en/knowledge-storage.md
title: "Canonical knowledge 저장"
summary: "선택된 Wiki backend 정본과 항상 disposable인 SQLite projection."
tags: [knowledge, sqlite]
aliases: ["Markdown SQLite 경계"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:9834a07f92cb41cb60c697f71aed30f8cc7874e338d51eff5a8a365a515a13e6"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:2e268d2a33c699c6b77a5c711df6a50eaf95624964dc616848bf29321de3624d"
links: [docs-wiki-architecture, host-external-integrations, shared-index]
reviewed_revision: "git:2b819c1060972bb2416a751ff17e596094b00a6b"
status: active
---

# Canonical knowledge 저장

Source knowledge·run·role·plan 정본: tracked Markdown·YAML·TOML. Consumer Markdown
mode 정본: Wiki Markdown. Notion mode 정본: selected Notion scope, active local Wiki
Markdown 0건. SQLite 역할: 재생성 가능한 local search projection. Durable fact의
유일한 저장소로 사용 금지.
