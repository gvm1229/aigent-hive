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
  - "repo:docs/decisions/ADR-0014-docs-wiki-architecture.md#sha256:ec0fe3e284ab7ea2effc9330f4a82918bc643c58aaae88866fc8af28e2be477f"
links: [knowledge-preservation, knowledge-storage]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Docs Wiki Architecture

`docs/` is one Wiki graph with a home, complete index, topic maps, human-readable
architecture and guides, and bilingual atomic facts under `docs/facts/`.
The former standalone source Wiki layout and name are absent from tracked source;
the current CLI, Skills, tests, and index use `docs/facts/`.
