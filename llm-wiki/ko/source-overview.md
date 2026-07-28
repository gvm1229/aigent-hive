---
schema_version: 1
pair_id: source-overview
topic_slug: source-overview
language: ko
counterpart: ../en/source-overview.md
title: "Source Workspace 개요"
summary: "Aigent Hive Rust source workspace의 목적, runtime 경계와 canonical identity."
tags: [architecture, provider-neutral, source-workspace]
aliases: ["Aigent Hive 소스"]
sources:
  - "repo:AGENTS.md#sha256:8293c7e01a78bbf6106fc6ee9cca9748171ba2361c5003883ad11faa4a81b396"
  - "repo:Cargo.toml#sha256:f9452a03c8e2ab1cb4e673a62cab0cd3aba3674d7b12175ad08df85fd56b5478"
  - "repo:hive-source.json#sha256:528b3c6a8f8614a38065144f2de9f3cd527474d5e4ec3f720acd6a27e60f2019"
links: [boundaries, crate-architecture, index]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Source Workspace 개요

Aigent Hive: Rust CLI와 provider-neutral agent harness source workspace. 이미 인증된 Codex,
Claude Code 또는 Antigravity-compatible host 위에서 deterministic local harness의 build,
test, package, install, update 수행.

- Hive 비소유: model-provider API 호출, provider credential, model execution
- Host 또는 compatible external orchestration layer 소유: model call, session, retry, subagent
- Hive 소유: local setup, projection, typed contract, canonical Markdown, index, validation,
  migration, update safety

`hive-source.json`: source identity와 `consumer_setup_allowed: false` 선언. 이 root를 대상으로
한 consumer setup 거부. Consumer artifact 위치: disposable test workspace 또는 독립 user
project. Source tree 내부 생성 금지.
