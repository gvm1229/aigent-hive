---
schema_version: 1
pair_id: knowledge-preservation
topic_slug: knowledge-preservation
language: en
counterpart: ../ko/knowledge-preservation.md
title: "Knowledge Preservation During Simplification"
summary: "Valid knowledge moves to a canonical locator before its original surface is shortened."
tags: [documentation, knowledge]
aliases: ["Move before delete"]
sources:
  - "repo:docs/decisions/ADR-0014-docs-wiki-architecture.md#sha256:99652573c72c2d45b969f8b406bd7a455956559da1253b19894b222a60a6ca59"
links: [docs-wiki-architecture, knowledge-storage]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Knowledge Preservation During Simplification

Before shortening a README, guide, overview, or Wiki page, every disappearing durable
claim needs a tracked replacement locator. Deletion is reserved for deprecated,
incorrect, or superseded knowledge.
