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
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:3e395404715ede791de670330489065939b0c81292a7897e30cdfacae601085c"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:5829f07a1626ec8adad1aad8bdbfd0ec25bc0fd6"
status: active
---

# 0.11.0 Foundation Refactor Scope

The maintainer selected 0.11.0-test.1 on top of the unreleased 0.10.4 changes, with no separate 0.10.4 release. Preserve existing features and commands. Improve knowledge capture, retrieval, and fresh-session continuity first; verify Codex before other hosts. The authorized task creates a branch and implementation plan. Product implementation and live acceptance remain future work; stable release requires separate version-specific approval.
