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
  - "repo:docs/plans/contracts/05-rust-boundaries.md#sha256:94c01e74d71356343b467917b0b1afe7669caf00f97d7b72de2fc98d872aa62a"
links: [artifact-boundaries, source-development]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Rust Crate Ownership

`hive-core` owns invariants, `hive-render` deterministic projection, `hive-wiki`
Markdown and SQLite knowledge, `hive-projection` Skill routing, `hive-update`
verification and migration, and `hive-cli` command adapters.
