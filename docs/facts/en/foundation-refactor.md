---
schema_version: 1
pair_id: foundation-refactor
topic_slug: foundation-refactor
language: en
counterpart: ../ko/foundation-refactor.md
title: "0.11.0 Foundation Refactor Scope"
summary: "The 0.11.0 refactor inherits unreleased 0.10.4 changes, preserves existing features, and verifies core knowledge flows on Codex first."
tags: [architecture, refactor, version]
aliases: []
sources:
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:8a61ed9ab2c5194bed2ccaad40ee31154be17dc4cfc6f006fbc68f2c2a50f487"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# 0.11.0 Foundation Refactor Scope

The maintainer selected 0.11.0-test.1 on top of the unreleased 0.10.4 changes, with no separate 0.10.4 release. Preserve existing features and commands. Improve knowledge capture, retrieval, and fresh-session continuity first; verify Codex before other hosts. Source branch enforcement is implemented. Host policy hooks join the refactor plan; their product implementation and live acceptance remain future work; stable release requires separate version-specific approval.
