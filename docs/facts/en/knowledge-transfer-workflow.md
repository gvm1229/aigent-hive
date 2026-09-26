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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:0ffab09ca1eaac47b41608e04003fdbc51ee7e5edbaf2386554c885d9f55ec58"
  - "repo:crates/hive-cli/src/knowledge_transfer.rs#sha256:d1a6df6babfbed54b46bb505889921a30fe86fd14fbd4cc0230d51bf7a99de92"
  - "repo:crates/hive-wiki/src/bundle_store.rs#sha256:74b41b0f34ce6ed9bb03f5dbb1a8760bc24e24da5667eb4a485c84a931ddacc0"
  - "repo:docs/guides/knowledge-transfer.md#sha256:18fcddede882c3dbcfa642b5b6c2b6be6e4bac898532e03c3f178da56c8633af"
  - "repo:harness/skills/knowledge-transfer/SKILL.md#sha256:7b4bbe52c0e4af139f61ded9ba5c75562d21c8e0011530af6512563bbaea7188"
links: [global-knowledge-bundle-transfer, knowledge-storage]
reviewed_revision: "git:dd63333a702a7a89585d101d2b9d043ebd0987d8"
status: active
---

# Knowledge transfer workflow

`knowledge-transfer` moves existing Markdown through `.hivekb`; `knowledge-scan` extracts new knowledge. Multi-bundle preview removes exact duplicates and returns input digests, semantic candidates, and same-path variants. Reviewed `separate`, `equivalent`, or `choose` binds one canonical apply. Collapsed and unselected Wiki originals remain portable merge provenance outside active search. Private collections remain detached until authorized attachment. FTS and optional vector decisions remain separate from canonical transfer.
