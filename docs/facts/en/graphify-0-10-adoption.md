---
schema_version: 1
pair_id: graphify-0-10-adoption
topic_slug: graphify-0-10-adoption
language: en
counterpart: ../ko/graphify-0-10-adoption.md
title: "Graphify 0.10 Adoption Decision"
summary: "The full Graphify knowledge-graph scope is excluded; a code-only optional adapter proposal awaits 0.10.0 scope approval."
tags: [graphify, knowledge, security, v0-10]
aliases: ["Graphify adoption", "knowledge graph decision"]
sources:
  - "repo:docs/plans/backlog/graphify-knowledge-graph.md#sha256:1558ec29827e73f622ce5f978f07e8f800c12600b1efbe75275e2ef072096431"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:63ec71461c610b4ccab8e186d8337f504b28ca5cd1a25dcd793872e7960bb427"
  - "repo:docs/research/graphify-0.10-feasibility.md#sha256:6bc52a7a6fa89601c5b20d851cb721e6b0f5d0e59b51b6d18963baaa69b6930e"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:daa32013cd5c9f506551532d0a5692d5644aeeaf"
status: active
---

# Graphify 0.10 Adoption Decision

Graphify `0.9.47` passed repeated full code-graph builds and small Windows query performance.
Incremental updates diverged from full regeneration, and the upstream global graph lacks Hive
collection isolation. The full knowledge-graph scope remains excluded. The current proposal uses
Hive-native Markdown relationships plus an optional full-rebuild Graphify code-only adapter.
`SCP10-001` scope approval is pending. No product integration has occurred.
