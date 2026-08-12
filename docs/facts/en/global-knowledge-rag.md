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
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:196117cadc85737e0dbe35c8fcc6699e5180632d919782c2312453f588b3ab7a"
  - "repo:docs/plans/active/v0.9.0-global-knowledge-rag.md#sha256:6049186f49dae584b981a8bb888ba15f43e7f61e085247f04b546ef368f7f6ce"
  - "repo:docs/plans/active/v0.9.0-knowledge-autocapture-regression.md#sha256:7fe35e6b1bdba462121104d9db09874ee755aa5c7ad65c85031353e35a172f0d"
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
