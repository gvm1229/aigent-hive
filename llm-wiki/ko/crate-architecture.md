---
schema_version: 1
pair_id: crate-architecture
topic_slug: crate-architecture
language: ko
counterpart: ../en/crate-architecture.md
title: "Rust Crate 구조"
summary: "Workspace member, CLI command adapter, Wiki core 재사용과 release verifier 경계."
tags: [architecture, crates, rust]
aliases: ["Rust 워크스페이스 구조"]
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

# Rust Crate 구조

Root workspace member 6개: `hive-cli`, `hive-core`, `hive-projection`, `hive-render`,
`hive-update`, `hive-wiki`. 모든 member의 공통 lint 경계: unsafe Rust 금지, Clippy `all`과
`pedantic` warning policy.

`hive-cli` 역할: top-level command adapter. Entrypoint에서 setup, user installation, Source
Wiki, consumer knowledge, project upgrade, index, routing, prompt, hook, usage, role, run, judge,
release와 update command를 전용 module로 전달.

`hive-wiki` manifest dependency: `hive-core`, capability-scoped filesystem access,
serialization과 SQLite. Source Wiki 결정의 재사용 범위: Markdown parser, lint, index와 query
core. 재사용 제외: installed consumer layout, runtime, approval과 knowledge state.

Release·update 경계: verifier-only. Hive runtime 제외 대상: model-provider API, downloader,
package-manager execution, release signing과 private-key custody.
