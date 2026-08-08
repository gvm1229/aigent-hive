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
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:bf987da6a220df4aa4194f87928626ea8321438671c9d4369c8e097fd272c8ec"
links: [docs-wiki-architecture, host-external-integrations, shared-index]
reviewed_revision: "git:907817827a6733dd380aaedea2e7592bc10a7311"
status: active
---

# Canonical knowledge 저장

Source knowledge·run·role·plan 정본: tracked Markdown·YAML·TOML. Consumer Markdown
mode 정본: Wiki Markdown. Notion mode 정본: selected Notion scope, active local Wiki
Markdown 0건. SQLite 역할: 재생성 가능한 local search projection. Durable fact의
유일한 저장소로 사용 금지.
