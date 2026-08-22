---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: en
counterpart: ../ko/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 Product Scope"
summary: "The 0.10.0 scope combines relationship search, a conditional hybrid-vector gate, nested-project scanning, Skill reservations, safe upgrade, and release qualification."
tags: [knowledge, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:1645eb2249265b75d27b0c65a709806f4999a0ec425e8e874336bcda084b702c"
  - "repo:docs/decisions/product-release-decisions.md#sha256:25bd2880270b2dd21bf09d5efe576f4164b8d02fadd8366f8649d8d50d38bded"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
status: active
---

# Aigent Hive 0.10.0 Product Scope

The final scope includes Hive-native Markdown relationships, optional full-rebuild Graphify
code extraction, FTS and relation routing, a conditional hybrid-vector gate, metadata-first
retrieval, lifecycle and drift evidence, nested-project scanning, host-owned Skill reservations,
safe upgrade, and three-OS release qualification. Stable publication also requires explicit
maintainer approval. Natural continuation can route complex work to `verified-workflow`, and an
explicit `adversarial-judge` step can request clean-context host judging. A failed vector gate adds
no dependency.
