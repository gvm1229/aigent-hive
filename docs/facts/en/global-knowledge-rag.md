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
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-global-knowledge-rag.md#sha256:6049186f49dae584b981a8bb888ba15f43e7f61e085247f04b546ef368f7f6ce"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:fcbeb8013ecc51ea78bb0087d0172372840585b7878f3f72c3fdf0b74b805080"
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
