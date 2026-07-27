---
schema_version: 1
pair_id: knowledge
topic_slug: knowledge
language: en
counterpart: ../ko/knowledge.md
title: "Knowledge and Index Architecture"
summary: "Canonical Markdown ownership, disposable SQLite projections, and separation of source and consumer knowledge."
tags: [knowledge, markdown, sqlite]
aliases: ["knowledge architecture"]
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

# Knowledge and Index Architecture

Durable Hive knowledge is tracked Markdown. SQLite provides local FTS, tag, alias, backlink,
source, and ranking projections; it never owns a durable fact and may be deleted and rebuilt
without a model call or network request.

Consumer projects keep canonical knowledge inside their installed `.hive/knowledge/` tree. An
explicit, policy-checked promotion workflow may copy only eligible project-neutral facts,
preferences, or portable workflows into the separate user-root knowledge store. Confidential
categories, credentials, private paths, and excluded project sources fail closed.

The source Wiki is a different knowledge class. Its English and Korean pages are exact pairs under
`llm-wiki/`, cite digest-bound repository files, and rebuild an ignored index under
`.agents/work/source-wiki/`. Consumer runtime state and consumer knowledge are never imported into
this source corpus.
