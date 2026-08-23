---
schema_version: 1
pair_id: graphify-0-10-adoption
topic_slug: graphify-0-10-adoption
language: en
counterpart: ../ko/graphify-0-10-adoption.md
title: "Graphify 0.10 Adoption Decision"
summary: "0.10.0 approves Hive-native Markdown relationships and an optional full-rebuild Graphify code-only adapter; the full Graphify knowledge graph remains excluded."
tags: [graphify, knowledge, security, v0-10]
aliases: ["Graphify adoption", "knowledge graph decision"]
sources:
  - "repo:docs/plans/active/knowledge-relationship-graph-0.10.0.md#sha256:cb76dcf1ca263a1fb5f0f7e0c030decd04c4dd65207980cb0698f1451848109c"
  - "repo:docs/plans/backlog/graphify-knowledge-graph.md#sha256:6ab392f6613412116a8fc24ad447236f319ceea7dee257ace138a300fc3cf960"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:docs/research/graphify-0.10-feasibility.md#sha256:a86ab9852f0fbd1a1737c97fcd236cf140fdc194e1f00d0e213e213a6f2fe600"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
status: active
---

# Graphify 0.10 Adoption Decision

Graphify `0.9.47` passed repeated full code-graph builds and small Windows query performance.
Incremental updates diverged from full regeneration, and the upstream global graph lacks Hive
collection isolation. The full knowledge-graph scope remains excluded. The approved `0.10.0`
scope uses Hive-native Markdown relationships plus an optional full-rebuild Graphify code-only
adapter. The adapter now requires exact consent, platform wheel lock, code-only receipt, grounded
locators, atomic generation activation, and native fallback. Cross-platform public acceptance
remains pending.
