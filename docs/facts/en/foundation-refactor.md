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
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:f548166391661d9103bf724f42c96bf073c9a4ba947103932b352806544ecfcd"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:23a94c06874fd65f8cdb0151f8512caed88fd792"
status: active
---

# 0.11.0 Foundation Refactor Scope

The maintainer selected 0.11.0-test.1 on top of the unreleased 0.10.4 changes, with no separate 0.10.4 release. Preserve existing features and commands. Improve knowledge capture, retrieval, and fresh-session continuity first; verify Codex before other hosts. Source branch enforcement is implemented. Host policy hooks join the refactor plan; their product implementation and live acceptance remain future work; stable release requires separate version-specific approval.
