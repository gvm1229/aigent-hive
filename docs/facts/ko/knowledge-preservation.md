---
schema_version: 1
pair_id: knowledge-preservation
topic_slug: knowledge-preservation
language: ko
counterpart: ../en/knowledge-preservation.md
title: "간소화 과정의 knowledge 보존"
summary: "원래 surface 축약 전 valid knowledge의 canonical locator 이동."
tags: [documentation, knowledge]
aliases: ["삭제 전 이동"]
sources:
  - "repo:docs/decisions/ADR-0014-docs-wiki-architecture.md#sha256:99652573c72c2d45b969f8b406bd7a455956559da1253b19894b222a60a6ca59"
links: [docs-wiki-architecture, knowledge-storage]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# 간소화 과정의 knowledge 보존

README·guide·overview·Wiki page 축약 전 사라질 durable claim별 tracked replacement
locator 확보. 삭제 허용 범위: deprecated·incorrect·superseded knowledge.
