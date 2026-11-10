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
  - "repo:docs/architecture/policy-rule-inventory.md#sha256:bc8d431af65678302984520646b8b5b4775a915714fdde21c42a4b863a0ef32a"
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:e609abd004c690bcdce2822313645ba7dd733fa9114e778c6305314c4ada0813"
links: [artifact-boundaries, knowledge-source-freshness, orchestration-ownership]
reviewed_revision: "git:dd4e2773e98d7acb13fa0dc77ad11167faaf23fb"
status: active
---

# 0.11.0 Foundation Refactor Scope

0.11.0-test.1 inherits unreleased 0.10.4 changes without a separate release. Preserve features and commands; improve knowledge flows and verify Codex first. Approved exception: automatic resume requires the current session and positive process ID. Seven approved proposals cover failure classification, result aggregation, effective diagnostics, held-out evaluation, stable-context reuse, reviewed improvement candidates, and context freshness. HK-005 owns review; existing HK criteria and RFK-002 own the rest. Source branch checks, native file-hook adapters, explicit configuration and run candidate review are implemented. Claude and periodic auto-stop are future work; qualify Codex/Antigravity. Stable release and user installation need separate approval.
