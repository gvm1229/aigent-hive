---
schema_version: 1
pair_id: crate-ownership
topic_slug: crate-ownership
language: en
counterpart: ../ko/crate-ownership.md
title: "Rust Crate Ownership"
summary: "Each workspace crate owns one provider-neutral implementation boundary."
tags: [architecture, rust]
aliases: ["Crate map"]
sources:
  - "repo:docs/archive/plans/foundations/contracts/05-rust-boundaries.md#sha256:ad974c7d93341ff4e8cde0fbc0801b155e6530bdc690f86bcec5679d18acfae9"
links: [artifact-boundaries, source-development]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# Rust Crate Ownership

`hive-core` owns invariants and the planned provider-neutral orchestration reducer
and logical scheduler. `hive-render` owns deterministic projection, `hive-wiki`
Markdown and SQLite knowledge, `hive-projection` Skill routing, `hive-update`
verification and migration, and `hive-cli` command adapters.
