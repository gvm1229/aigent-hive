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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:fe1c6e487af1fff9bfd8f5d7aa8c2b3ba7003fe0e3ceeba747c9bcd4ac972b30"
  - "repo:crates/hive-cli/src/knowledge_transfer.rs#sha256:d1a6df6babfbed54b46bb505889921a30fe86fd14fbd4cc0230d51bf7a99de92"
  - "repo:crates/hive-wiki/src/bundle_store.rs#sha256:5dae98a4757bee2d7025ac3ec711e31ae4a33200b04d9b5aaad8746267a97e89"
  - "repo:docs/guides/knowledge-transfer.md#sha256:f0f48767f5fcb83450853ddf95d0c9f3f573b3c2fb3e0b626adb5af5ed742bf1"
  - "repo:harness/skills/knowledge-manage/references/transfer.md#sha256:67ae49affca4cbd70567fc92eb0f06f105be2ad1d728b1bfad5c783dfc82835f"
links: [global-knowledge-bundle-transfer, knowledge-storage]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Knowledge transfer workflow

`knowledge-transfer` moves existing Markdown through `.hivekb`; `knowledge-scan` extracts new knowledge. Multi-bundle preview removes exact duplicates and returns input digests, semantic candidates, and same-path variants. Reviewed `separate`, `equivalent`, or `choose` binds one canonical apply. Collapsed and unselected Wiki originals remain portable merge provenance outside active search. Private collections remain detached until authorized attachment. FTS and optional vector decisions remain separate from canonical transfer.
