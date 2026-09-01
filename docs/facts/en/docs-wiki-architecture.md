---
schema_version: 1
pair_id: docs-wiki-architecture
topic_slug: docs-wiki-architecture
language: en
counterpart: ../ko/docs-wiki-architecture.md
title: "Docs Wiki Architecture"
summary: "Current documentation, future backlog, and frozen history have separate Wiki lifecycles."
tags: [documentation, wiki]
aliases: ["Source docs Wiki"]
sources:
  - "repo:docs/archive/README.md#sha256:4fa687e5a3603890bca9e557df8ad8e80de9f87eafa76d18e7bdf11c827eef6f"
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:c9e698b54b31db5561a9b3611164ebc2d851bd7fa92087161864ea2092801b93"
  - "repo:docs/decisions/ADR-0014-docs-wiki-architecture.md#sha256:ec0fe3e284ab7ea2effc9330f4a82918bc643c58aaae88866fc8af28e2be477f"
  - "repo:docs/plans/README.md#sha256:85944730779c8686d4f436fe735f8e65b0ee34f8e5dee048103a8e85cd3f508a"
links: [knowledge-preservation, knowledge-storage]
reviewed_revision: "git:41f05a55741e319594e5f7ffe811e0e623ade499"
status: active
---

# Docs Wiki Architecture

`docs/` is one Wiki graph with a home, complete index, topic maps, human-readable
architecture and guides, and bilingual atomic facts under `docs/facts/`.
The former standalone source Wiki layout and name are absent from tracked source;
the current CLI, Skills, tests, and index use `docs/facts/`.
Automatic source-workspace lookup uses `hive source-wiki query` after detecting
`hive-source.json`; consumer `hive knowledge retrieve` is limited to an attached external project.
Current execution loads `PLAN.md`, `CURRENT.md`, and only the owning active fragment.
Version-unbound candidates stay in `docs/plans/backlog/`; completed or superseded records stay in
`docs/archive/` and are excluded from automatic task context and current-document checks.
