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
  - "repo:Cargo.toml#sha256:f9452a03c8e2ab1cb4e673a62cab0cd3aba3674d7b12175ad08df85fd56b5478"
  - "repo:crates/hive-cli/src/main.rs#sha256:ca575035f04f905bf2ca8f7d780589b5513f7ed672b4de4d9aa82624fda68340"
  - "repo:crates/hive-wiki/Cargo.toml#sha256:25c10369a5e5b77938ddc7d39541be4cf647d7c051696d9fe230de4a4ef9e107"
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:15dbcb1c9e294078dc641d0c51c3655bd047cdf1c57629cb4158e7d047097f1b"
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
