---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: en
counterpart: ../ko/global-knowledge-rag.md
title: "Global Knowledge RAG"
summary: "v0.9 operational guidance now enforces automatic every-turn capture; fresh Windows Codex acceptance and release qualification remain pending."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:2dece311aef55de6a52b9f3f8f79fbf928009f312a98d7ab0c3cb09cfa9db741"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6763857d275d0a35065e27f744e4a7d2c83d77b876abcdb5343f37be01ffe35e"
  - "repo:docs/plans/active/v0.9.0-knowledge-autocapture-regression.md#sha256:1e814cdd5a4f4f0806e2dab7789d7dd2ffd4df86d55256cbe292880e3d44e7b7"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:2234885542a2c3e82514121b890e129b89e5e563"
status: active
---

# Global Knowledge RAG

Canonical remember, retrieval, and idempotency work. Generated English and Korean user
guidance now requires scope classification, exactly one safe `remember` call, and Markdown/index
receipt; disabled Wiki guidance rejects writes. All-host projection and localized Skill metadata
preserve the automatic route. Fresh Windows Codex write-and-recall acceptance and replacement
stable qualification remain release gates.
