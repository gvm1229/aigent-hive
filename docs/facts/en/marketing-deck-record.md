---
schema_version: 1
pair_id: marketing-deck-record
topic_slug: marketing-deck-record
language: en
counterpart: ../ko/marketing-deck-record.md
title: "Marketing Deck Record"
summary: "A tracked handoff stores the safe locator and resumption criteria for the external deck."
tags: [artifact, marketing]
aliases: ["LumaDeck handoff"]
sources:
  - "repo:docs/state/artifacts/aigent-hive-marketing-deck.md#sha256:14498376f97d611f701aad80e11ab76d7b3e4f5204203d10391edea94c6a48d9"
links: [product-purpose, v0-9-skill-suite-plan]
reviewed_revision: "git:2b819c1060972bb2416a751ff17e596094b00a6b"
status: active
---

# Marketing Deck Record

The external LumaDeck artifact is a 91-slide, 60-minute `aigent-hive-overview` presentation targeting
stable `0.9.0`. It covers feature-and-example pairs for all 22 public short Skill names, stable
installation choices, and implementation principles plus the repository's planning, ADR, prefix,
workflow, and verification conventions. The deck uses Pretendard Variable throughout, applies equal
48-pixel outer padding on every side at 1280×720, and passed font, line-height, text-box contact,
overflow, production-build, and Safari visual checks. The source corpus keeps only its safe locator,
scope, version basis, verification result, and resumption conditions.
