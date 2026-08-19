---
schema_version: 1
pair_id: graphify-0-10-adoption
topic_slug: graphify-0-10-adoption
language: en
counterpart: ../ko/graphify-0-10-adoption.md
title: "Graphify 0.10 Adoption Decision"
summary: "Graphify 0.9.47 is excluded from the 0.10.0 product scope after failing incremental equivalence and knowledge-visibility isolation hard gates."
tags: [graphify, knowledge, security, v0-10]
aliases: ["Graphify adoption", "knowledge graph decision"]
sources:
  - "repo:docs/plans/backlog/graphify-knowledge-graph.md#sha256:1558ec29827e73f622ce5f978f07e8f800c12600b1efbe75275e2ef072096431"
  - "repo:docs/research/graphify-0.10-feasibility.md#sha256:6bc52a7a6fa89601c5b20d851cb721e6b0f5d0e59b51b6d18963baaa69b6930e"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:87c28dd940a946737a32bc484220de340b50e3ad"
status: active
---

# Graphify 0.10 Adoption Decision

Graphify `0.9.47` passed repeated full code-graph builds and small Windows query performance.
Incremental updates diverged from full regeneration on the same source, and the single upstream
global graph lacks collection visibility isolation. Product integration for `0.10.0` stopped;
the candidate remains in the version-independent backlog.
