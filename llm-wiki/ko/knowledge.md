---
schema_version: 1
pair_id: knowledge
topic_slug: knowledge
language: ko
counterpart: ../en/knowledge.md
title: "Knowledge와 Index 구조"
summary: "Canonical Markdown ownership, disposable SQLite projection과 source-consumer knowledge 분리."
tags: [knowledge, markdown, sqlite]
aliases: ["지식 구조"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:8bfd86a2ede49c3ce92f0a8e57a06c922c19248627d7d3552dd1777c1ee4954b"
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:3589ba7f2032870f8d63346312cc0f5358700934de3cf6a84602bf3397cff801"
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:e5315d16b0dc932bcedc79add82460220c64bec84e5f1e30e2ed672c93eaa5d4"
  - "repo:harness/directives/01-project-knowledge.md#sha256:a1809cc9a66646c7f10ea5fcee490c985998e110abf8b8855d23f0b53c12ae56"
  - "repo:schemas/source-wiki-page.schema.json#sha256:756201e9cd032de33460357516d0bc3176d837551163754583ded2e3338d3643"
links: [boundaries, crate-architecture, skill-routing]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Knowledge와 Index 구조

Durable Hive knowledge의 정본: tracked Markdown. SQLite의 역할: local FTS, tag, alias,
backlink, source와 ranking projection. SQLite에만 존재하는 durable fact 금지. Model call과
network 없이 삭제·재구축 가능한 index 계약.

Consumer project의 canonical knowledge 위치: installed `.hive/knowledge/`. 별도 explicit
policy review를 통과한 project-neutral fact, preference 또는 portable workflow만 user-root
knowledge store로 promotion 가능. Confidential category, credential, private path와 excluded
project source는 fail-closed.

Source Wiki는 별도 knowledge class. `llm-wiki/` 아래 English·Korean exact pair, digest-bound
repository source citation, `.agents/work/source-wiki/` 아래 ignored index. Consumer runtime
state와 consumer knowledge의 source corpus import 금지.
