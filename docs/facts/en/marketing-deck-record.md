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
  - "repo:docs/state/artifacts/aigent-hive-marketing-deck.md#sha256:ffc5ef8f86100b42662a789e6f93878ab7f6b09f60bc1f6231b9306cdcd3e1ba"
links: [product-purpose, test-distribution, v0-9-skill-suite-plan]
reviewed_revision: "git:c949c754ccb602f10468ae30bb3e402e4e01f39d"
status: active
---

# Marketing Deck Record

The external LumaDeck artifact is a 91-slide, 60-minute `aigent-hive-overview` presentation based on
`0.9.0-test.5`. It covers feature-and-example pairs for all 22 public short Skill names, README-based
installation choices, and implementation principles plus the repository's planning, ADR, prefix,
workflow, and verification conventions. The deck has embedded notes and a separate presenter script,
and its production build and all 91 slides passed 1280×720 overflow verification. The source corpus
keeps only its safe locator, scope, version basis, verification result, and resumption conditions.
