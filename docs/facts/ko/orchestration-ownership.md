---
schema_version: 1
pair_id: orchestration-ownership
topic_slug: orchestration-ownership
language: ko
counterpart: ../en/orchestration-ownership.md
title: "Orchestration ownership"
summary: "Compatible OMX·OMC 우선, 그 외 host-native owner."
tags: [orchestration, ownership]
aliases: ["Orchestration owner"]
sources:
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:0888b22473297bf6161141b508e7e276d5c8cc3bf5ffe9c43269b16c3fec347e"
links: [product-non-goals, skill-routing]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Orchestration ownership

Owner 우선순위: Codex의 compatible OMX, Claude의 compatible OMC, 그 외 active
host의 truthful native capability. Pinned run owner의 silent switch 금지.
