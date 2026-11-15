---
schema_version: 1
pair_id: knowledge-transfer-workflow
topic_slug: knowledge-transfer-workflow
language: en
counterpart: ../ko/knowledge-transfer-workflow.md
title: "Knowledge transfer workflow"
summary: "Single and multi-bundle imports bind exact inputs, destination bytes, review decisions, and optional vector deferral."
tags: [knowledge, portability]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:6b4d501127b0ba69d8d40d07cb57da9992ca34380d7c728f17e39deb9b6b11ee"
  - "repo:crates/hive-cli/src/knowledge_transfer.rs#sha256:d1a6df6babfbed54b46bb505889921a30fe86fd14fbd4cc0230d51bf7a99de92"
  - "repo:crates/hive-wiki/src/bundle_store.rs#sha256:f2004da148a9e74f922c4911266f8835853044d52601d40f6d453bfa9add5848"
  - "repo:docs/guides/knowledge-transfer.md#sha256:ca83f47aee9746c7d73acaae8d9bb316a1d625036a859db4dc00833e42fdf263"
  - "repo:harness/skills/knowledge-transfer/SKILL.md#sha256:7b4bbe52c0e4af139f61ded9ba5c75562d21c8e0011530af6512563bbaea7188"
links: [global-knowledge-bundle-transfer, knowledge-storage]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Knowledge transfer workflow

`knowledge-transfer` moves existing Markdown through `.hivekb`; `knowledge-scan` extracts new knowledge. Multi-bundle preview removes exact duplicates and returns input digests, semantic candidates, and same-path variants. Reviewed `separate`, `equivalent`, or `choose` binds one canonical apply. Collapsed and unselected Wiki originals remain portable merge provenance outside active search. Private collections remain detached until authorized attachment. FTS and optional vector decisions remain separate from canonical transfer.
