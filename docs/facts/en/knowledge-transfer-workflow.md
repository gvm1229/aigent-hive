---
schema_version: 1
pair_id: knowledge-transfer-workflow
topic_slug: knowledge-transfer-workflow
language: en
counterpart: ../ko/knowledge-transfer-workflow.md
title: "Knowledge transfer workflow"
summary: "Archive and destination digests bind imports; vector deferral preserves the user's preference."
tags: [knowledge, portability]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge_transfer.rs#sha256:d1a6df6babfbed54b46bb505889921a30fe86fd14fbd4cc0230d51bf7a99de92"
  - "repo:crates/hive-wiki/src/bundle_store.rs#sha256:46f4d198668dc35e687d07331e7eaaed304d6d99d23582e85ba110331141ed34"
  - "repo:docs/guides/knowledge-transfer.md#sha256:20f057772eec3f009864ab41e104bb265ad5171f3344d826cb2892551a0882b9"
links: [global-knowledge-bundle-transfer, knowledge-storage]
reviewed_revision: "git:523892f0009d7ee04af9381981cb41ba01c4045d"
status: active
---

# Knowledge transfer workflow

`knowledge-transfer` moves existing Markdown through `.hivekb`; `knowledge-scan` extracts new knowledge. Apply binds the sender's archive SHA-256 and the reviewed destination bytes under the writer lock. Explicit conflict exclusion preserves local files. Imported private collections remain detached until authorized attachment. FTS readiness is separate from optional vector work. Transfer receipts retain no/cancel/yes semantics without changing global vector preferences. Rebuild uses only imported, previously enabled partitions; missing runtime or scope approval never triggers installation.
