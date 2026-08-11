---
schema_version: 1
pair_id: global-knowledge-rag
topic_slug: global-knowledge-rag
language: en
counterpart: ../ko/global-knowledge-rag.md
title: "Global Knowledge RAG"
summary: "v0.9.0-test.16 now ships automatic every-turn capture and is installed on Windows; fresh Windows Codex acceptance and stable qualification remain pending."
tags: [knowledge, rag, retrieval, v0-9]
aliases: ["Cross-project RAG", "Mandatory memory"]
sources:
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:2dece311aef55de6a52b9f3f8f79fbf928009f312a98d7ab0c3cb09cfa9db741"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6763857d275d0a35065e27f744e4a7d2c83d77b876abcdb5343f37be01ffe35e"
  - "repo:docs/plans/active/v0.9.0-knowledge-autocapture-regression.md#sha256:2438fc16f6ee5fc077d56f771755b1ade117d704c766b06765298d5362a92c86"
links: [knowledge-portability-scan, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:2234885542a2c3e82514121b890e129b89e5e563"
status: active
---

# Global Knowledge RAG

Canonical remember, retrieval, and idempotency work. `0.9.0-test.16` ships the generated English
and Korean guidance, Copier projection, plugin metadata, and catalog description requiring scope
classification, exactly one safe `remember` call, and a Markdown/index receipt; disabled Wiki
guidance rejects writes. Windows user installation and static installed guidance passed. Fresh
Windows Codex write-and-recall acceptance and replacement stable qualification remain release gates.
