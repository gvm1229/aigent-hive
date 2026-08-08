---
schema_version: 1
pair_id: knowledge-storage
topic_slug: knowledge-storage
language: ko
counterpart: ../en/knowledge-storage.md
title: "Canonical knowledge 저장"
summary: "v0.9 Markdown 정본과 항상 disposable인 SQLite projection. Notion은 v0.10 시험판 보류."
tags: [knowledge, sqlite]
aliases: ["Markdown SQLite 경계"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:9834a07f92cb41cb60c697f71aed30f8cc7874e338d51eff5a8a365a515a13e6"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
links: [docs-wiki-architecture, host-external-integrations, shared-index]
reviewed_revision: "git:2b819c1060972bb2416a751ff17e596094b00a6b"
status: active
---

# Canonical knowledge 저장

Source knowledge·run·role·plan 정본: tracked Markdown·YAML·TOML. v0.9 Consumer Wiki
정본: local Markdown. SQLite 역할: 재생성 가능한 local search projection. Durable fact의
유일한 저장소 사용 금지. Notion 정본은 첫 v0.10 시험판까지 보류.
