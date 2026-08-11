---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: en
counterpart: ../ko/global-knowledge-rag.md
title: "Global Knowledge RAG"
summary: "v0.9 implements the durable-memory store, but the operational user guidance omits the mandatory every-turn write gate."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:2dece311aef55de6a52b9f3f8f79fbf928009f312a98d7ab0c3cb09cfa9db741"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6763857d275d0a35065e27f744e4a7d2c83d77b876abcdb5343f37be01ffe35e"
  - "repo:docs/plans/active/v0.9.0-knowledge-autocapture-regression.md#sha256:5cddc7bd9f4ab4c1b868d8a6a86cf155503d008ff901276158e64afc447c841a"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:2234885542a2c3e82514121b890e129b89e5e563"
status: active
---

# Global Knowledge RAG

Canonical remember, retrieval, and idempotency work. The `0.9.0-test.13` Windows Codex
user guidance nevertheless omits the every-turn command and receipt, while localized
Skill metadata loses mandatory routing. Unregistered repositories can therefore skip
reusable facts. Existing validation proves byte equality, not this semantic gate.
`KAC-001–008` blocks stable publication until guidance, routing, semantic tests, and a
fresh Windows Codex write-and-recall pass without manual knowledge invocation.
