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
  - "repo:docs/plans/active/knowledge-relationship-graph-0.10.0.md#sha256:b9318a1ba4a61dde7bfed21e6e5915bcee0f4ab8e9f4be39cfdfddf01e280ec2"
  - "repo:docs/plans/backlog/graphify-knowledge-graph.md#sha256:6ab392f6613412116a8fc24ad447236f319ceea7dee257ace138a300fc3cf960"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:docs/research/graphify-0.10-feasibility.md#sha256:6bc52a7a6fa89601c5b20d851cb721e6b0f5d0e59b51b6d18963baaa69b6930e"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:d4e2cb66f2363efa84a18cebb7ff3de32dff91cf"
status: active
---

# Graphify 0.10 Adoption Decision

Graphify `0.9.47` passed repeated full code-graph builds and small Windows query performance.
Incremental updates diverged from full regeneration, and the upstream global graph lacks Hive
collection isolation. The full knowledge-graph scope remains excluded. The approved `0.10.0`
scope uses Hive-native Markdown relationships plus an optional full-rebuild Graphify code-only
adapter. Implementation and qualification remain pending.
