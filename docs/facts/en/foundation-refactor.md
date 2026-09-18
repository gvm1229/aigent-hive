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
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:4052a4c14e26af16fefcd94116c8060f351ccb0fe5fe8321d127c59ff0d517a7"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:568f9540dc893c089dbd72244d7b0c0608518eeb"
status: active
---

# 0.11.0 Foundation Refactor Scope

0.11.0-test.1 inherits unreleased 0.10.4 changes without a separate 0.10.4 release. Preserve features and commands; improve core knowledge flows and verify Codex first. Source branch enforcement is implemented. Seven accepted proposals cover failure classification, result aggregation, effective diagnostics, held-out evaluation, stable-context reuse, reviewed improvement candidates, and context freshness. HK-005 owns candidate review; existing HK criteria and RF-K02 own the rest. Product hooks and live acceptance remain pending. Stable release requires version-specific approval.
