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
  - "repo:Cargo.toml#sha256:f9452a03c8e2ab1cb4e673a62cab0cd3aba3674d7b12175ad08df85fd56b5478"
  - "repo:crates/hive-cli/src/main.rs#sha256:ca575035f04f905bf2ca8f7d780589b5513f7ed672b4de4d9aa82624fda68340"
  - "repo:crates/hive-wiki/Cargo.toml#sha256:25c10369a5e5b77938ddc7d39541be4cf647d7c051696d9fe230de4a4ef9e107"
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:15dbcb1c9e294078dc641d0c51c3655bd047cdf1c57629cb4158e7d047097f1b"
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
