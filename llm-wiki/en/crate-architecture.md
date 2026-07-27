---
schema_version: 1
pair_id: crate-architecture
topic_slug: crate-architecture
language: en
counterpart: ../ko/crate-architecture.md
title: "Rust Crate Architecture"
summary: "Workspace members, the CLI command adapter, Wiki-core reuse, and the release verifier boundary."
tags: [architecture, crates, rust]
aliases: ["Rust workspace architecture"]
sources:
  - "repo:Cargo.toml#sha256:ee731c226fdb29253df5f7fb1111573a892d1da34b38fd424e5ec7199f0f346a"
  - "repo:crates/hive-cli/src/main.rs#sha256:43d4302899e96e74a0f0e1b7f0b66a1e8b16e6c6e74e28d2d90e5c2c993e1ff3"
  - "repo:crates/hive-wiki/Cargo.toml#sha256:25c10369a5e5b77938ddc7d39541be4cf647d7c051696d9fe230de4a4ef9e107"
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:e5315d16b0dc932bcedc79add82460220c64bec84e5f1e30e2ed672c93eaa5d4"
links: [knowledge, source-overview, upgrade]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Rust Crate Architecture

The root workspace declares six members: `hive-cli`, `hive-core`, `hive-projection`,
`hive-render`, `hive-update`, and `hive-wiki`. All members inherit the workspace ban on unsafe
Rust and the `all` plus `pedantic` Clippy warning policy.

`hive-cli` is the top-level command adapter. Its entrypoint routes setup, user installation,
Source Wiki, consumer knowledge, project upgrade, index, routing, prompt, hook, usage, role, run,
judge, release, and update commands to their dedicated modules.

The `hive-wiki` manifest binds the Wiki library to `hive-core`, capability-scoped filesystem
access, serialization, and SQLite. The Source Wiki decision permits reuse of its Markdown parser,
lint, index, and query core while excluding installed consumer layout, runtime, approval, and
knowledge state.

Release and update remain verifier-only. Hive runtime excludes model-provider APIs, downloaders,
package-manager execution, release signing, and private-key custody from that boundary.
