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
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:d180f7a9c22d525888e329e026a7b971e579f877c03dd9fee265967ab34cec69"
links: [product-non-goals, skill-routing, v0-9-skill-suite-plan]
reviewed_revision: "git:be5253bcbd0d9818333e5702d0ef9ce438ee4d62"
status: active
---

# Orchestration ownership

Owner 우선순위: Codex의 compatible OMX, Claude의 compatible OMC, 그 외 active
host의 truthful native capability. Pinned run owner의 silent switch 금지. ADR-0015의
v0.9 새 run 대상 host-native Skill 조합 제안과 기존 owner pin 보존.
