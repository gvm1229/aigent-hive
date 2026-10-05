---
schema_version: 1
pair_id: v0-11-2-scope
topic_slug: v0-11-2-scope
language: en
counterpart: ../ko/v0-11-2-scope.md
title: "0.12.0 Scope and Windows First"
summary: "Maintainer-approved scope for 0.12.0, with Windows first and macOS-specific acceptance last."
tags: [scope, v0-12-0, windows]
aliases: []
sources:
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:84a2fba7eacd840fdd0bac536595b58b6f73afb2381fea7c8cc971a3ae1f6247"
links: [knowledge-storage, marketing-deck-record, product-purpose]
reviewed_revision: "git:ad412a7b2aa6bba927f5f841e0e5161f4fb60488"
status: active
---

# 0.12.0 Scope and Windows First

The maintainer moved the approved 0.11.2 work to 0.12.0 on 2026-10-05. It includes usage control, host-owned delegation, remaining Graphify work and diagnostics. Only verified host features activate; others stay disabled or pending. Enabled relations run after capture and preserve saved knowledge on analysis failure.

Windows comes first and macOS last. Claude live validation and signing remain pending. Notion and the old presentation are retired. Logo and vectors are complete; engine comparisons stay references. Obsidian needs Markdown without extra plugins. Research does not prove live acceptance.

The added zero-quota safeguard defaults on, with a separate session opt-out. It gates new Hive work; active host interruption remains unverified.
