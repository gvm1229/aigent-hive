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
  - "repo:docs/architecture/policy-rule-inventory.md#sha256:77bd57357ba6cb08f84c1ee8908d8859514bd114951faf6abc92b67d7ef30920"
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:3bb46e15dfa2752a7dd13e8ecd1f3255ff7ea6532625533a2527d2c7357aeb1a"
links: [artifact-boundaries, knowledge-source-freshness, orchestration-ownership]
reviewed_revision: "git:1cdebf8aef9dde963f5aa88a36042186fbf37e80"
status: active
---

# 0.11.0 Foundation Refactor Scope

0.11.0-test.1 inherits unreleased 0.10.4 changes without a separate release. Preserve features and commands; improve knowledge flows and verify Codex first. Approved exception: automatic resume requires the current session and positive process ID. Seven approved proposals cover failure classification, result aggregation, effective diagnostics, held-out evaluation, stable-context reuse, reviewed improvement candidates, and context freshness. HK-005 owns review; existing HK criteria and RFK-002 own the rest. Source branch checks, native file-hook adapters, explicit configuration and run candidate review are implemented. Live acceptance covers Codex/Antigravity; Claude is future work. Stable release and user installation need separate approval.
