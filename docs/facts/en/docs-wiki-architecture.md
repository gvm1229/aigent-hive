---
schema_version: 1
pair_id: docs-wiki-architecture
topic_slug: docs-wiki-architecture
language: en
counterpart: ../ko/docs-wiki-architecture.md
title: "Docs Wiki Architecture"
summary: "Human topic documents and atomic facts form one docs-based Wiki."
tags: [documentation, wiki]
aliases: ["Source docs Wiki"]
sources:
  - "repo:docs/decisions/ADR-0014-docs-wiki-architecture.md#sha256:99652573c72c2d45b969f8b406bd7a455956559da1253b19894b222a60a6ca59"
links: [knowledge-preservation, knowledge-storage]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Docs Wiki Architecture

`docs/` is one Wiki graph with a home, complete index, topic maps, human-readable
architecture and guides, and bilingual atomic facts under `docs/facts/`.
