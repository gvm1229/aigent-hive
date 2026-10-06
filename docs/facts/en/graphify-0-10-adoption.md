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
  - "repo:docs/plans/active/knowledge-relationship-graph-0.10.0.md#sha256:f6efdd74ccfa6e9bbe2b675dde40b3c70b7dfa1b3f36cb0833c5bc65f0c52b90"
  - "repo:docs/plans/backlog/graphify-knowledge-graph.md#sha256:1e6b7d401f969efedd4786f06b3f0f7637f61c15b65b112dcabdf0021fdc1b99"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:docs/research/graphify-0.10-feasibility.md#sha256:9812c704b47db91f150291c1cf0c9ea9857c1ecd4153e19814f7385491e76898"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:e91331b5a498484c8abde81fe0989df1bccf8ea6"
status: active
---

# Graphify 0.10 Adoption Decision

Graphify `0.9.47` passed repeated full code-graph builds and small Windows query performance.
Incremental updates diverged from full regeneration, and the upstream global graph lacks Hive
collection isolation. The full knowledge-graph scope remains excluded. The approved `0.10.0`
scope uses Hive-native Markdown relationships plus an optional full-rebuild Graphify code-only
adapter. The adapter now requires exact consent, platform wheel lock, code-only receipt, grounded
locators, atomic generation activation, and native fallback. KRG10-016 completed public
acceptance on three operating systems for that code-only scope. On 2026-10-05 the maintainer
assigned the remaining full knowledge-graph work to 0.11.2 criteria GPH112-001 through GPH112-003.
