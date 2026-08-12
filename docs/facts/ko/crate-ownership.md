---
schema_version: 1
pair_id: crate-ownership
topic_slug: crate-ownership
language: ko
counterpart: ../en/crate-ownership.md
title: "Rust crate ownership"
summary: "Workspace crate별 provider-neutral 구현 경계."
tags: [architecture, rust]
aliases: ["Crate map"]
sources:
  - "repo:docs/plans/contracts/05-rust-boundaries.md#sha256:ad974c7d93341ff4e8cde0fbc0801b155e6530bdc690f86bcec5679d18acfae9"
links: [artifact-boundaries, source-development]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# Rust crate ownership

Ownership map: `hive-core` invariant·계획된 provider-neutral orchestration
reducer·logical scheduler, `hive-render` deterministic projection, `hive-wiki`
Markdown·SQLite knowledge, `hive-projection` Skill routing, `hive-update`
verification·migration, `hive-cli` command adapter.
