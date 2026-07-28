---
schema_version: 1
pair_id: source-overview
topic_slug: source-overview
language: en
counterpart: ../ko/source-overview.md
title: "Source Workspace Overview"
summary: "Purpose, runtime boundary, and canonical identity of the Aigent Hive Rust source workspace."
tags: [architecture, provider-neutral, source-workspace]
aliases: ["Aigent Hive source"]
sources:
  - "repo:AGENTS.md#sha256:8293c7e01a78bbf6106fc6ee9cca9748171ba2361c5003883ad11faa4a81b396"
  - "repo:Cargo.toml#sha256:f9452a03c8e2ab1cb4e673a62cab0cd3aba3674d7b12175ad08df85fd56b5478"
  - "repo:hive-source.json#sha256:528b3c6a8f8614a38065144f2de9f3cd527474d5e4ec3f720acd6a27e60f2019"
links: [boundaries, crate-architecture, index]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Source Workspace Overview

Aigent Hive is a Rust CLI and provider-neutral agent-harness source workspace. It builds, tests,
packages, installs, and updates deterministic local harness material for users who are already
authenticated to Codex, Claude Code, or an Antigravity-compatible host.

Hive does not call model-provider APIs, hold provider credentials, or own model execution. The
active host or a compatible external orchestration layer owns model calls, sessions, retries, and
subagents. Hive owns local setup, projection, typed contracts, canonical Markdown, indexing,
validation, migration, and update safety.

`hive-source.json` identifies this repository as source and sets `consumer_setup_allowed` to
`false`. Consumer setup must therefore refuse this root. Consumer artifacts belong in disposable
test workspaces or independent user projects, never in the source tree.
