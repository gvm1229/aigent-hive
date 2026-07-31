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
  - "repo:docs/plans/contracts/05-rust-boundaries.md#sha256:94c01e74d71356343b467917b0b1afe7669caf00f97d7b72de2fc98d872aa62a"
links: [artifact-boundaries, source-development]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Rust crate ownership

Ownership map: `hive-core` invariant, `hive-render` deterministic projection,
`hive-wiki` Markdown·SQLite knowledge, `hive-projection` Skill routing,
`hive-update` verification·migration, `hive-cli` command adapter.
