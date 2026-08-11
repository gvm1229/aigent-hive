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
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:24822777fdee6dec2272b659009913e69929aba5046d0858a9b745dec0e350c5"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:dbeb3ee4cd8fbc2ca363c2d32aaa092e7dc1d0f884851925811ed1d580b48f1f"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:2234885542a2c3e82514121b890e129b89e5e563"
status: active
---

# Global Knowledge RAG

The canonical `hive knowledge remember` write, derived retrieval, and idempotency
work. The `0.9.0-test.13` Windows Codex installation nevertheless omits the
every-turn classification, command, and receipt rule from operational user guidance;
its localized `knowledge-capture` description also loses the mandatory-route meaning.
Project guidance contains the rule, so unregistered projects and ordinary global
turns can silently skip reusable user facts. Existing validation proves expected-byte
integrity, not this semantic gate.
