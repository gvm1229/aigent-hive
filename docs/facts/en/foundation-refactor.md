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
  - "repo:docs/architecture/policy-rule-inventory.md#sha256:925d51703dc8db78bb15bc7598d18225766fae6aeb582fa31bd2c2a0ad58921d"
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:f811647803b6816a30e62ad03653f54700b7e53a4a876ec2802f83fde8332ca3"
links: [artifact-boundaries, orchestration-ownership]
reviewed_revision: "git:a1b96c88b4938c8771ccf635ffef0e8dcafa9ff1"
status: active
---

# 0.11.0 Foundation Refactor Scope

0.11.0-test.1 inherits unreleased 0.10.4 changes without a separate release. Preserve features and commands; improve knowledge flows and verify Codex first. Seven approved proposals cover failure classification, result aggregation, effective diagnostics, held-out evaluation, stable-context reuse, reviewed improvement candidates, and context freshness. HK-005 owns review; existing HK criteria and RFK-002 own the rest. Source branch checks, native file-hook adapters, explicit configuration and run candidate review are implemented. Live host acceptance remains pending. Stable release and user installation need separate approval.
