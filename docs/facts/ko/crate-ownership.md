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
  - "repo:docs/plans/contracts/05-rust-boundaries.md#sha256:b947e72402f5d858ed31f71cc95dba653fc915a5b96ea7fc338c87d418a9ec2c"
links: [artifact-boundaries, source-development]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# Rust crate ownership

Ownership map: `hive-core` invariant·계획된 provider-neutral orchestration
reducer·logical scheduler, `hive-render` deterministic projection, `hive-wiki`
Markdown·SQLite knowledge, `hive-projection` Skill routing, `hive-update`
verification·migration, `hive-cli` command adapter.
