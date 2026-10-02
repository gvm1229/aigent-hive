---
schema_version: 1
pair_id: codex-skill-delivery
topic_slug: codex-skill-delivery
language: en
counterpart: ../ko/codex-skill-delivery.md
title: "Codex Skill Delivery and Automatic Cleanup"
summary: "Codex plugin delivery removes verified duplicate copies during user and project updates."
tags: [codex, skill, update]
aliases: ["스킬 중복", "자동 정리"]
sources:
  - "repo:crates/hive-render/src/skill_delivery.rs#sha256:89274e89fafa3a325edf96917faa943f25bd8531675c5a0ac3fec50fa4b09ecf"
  - "repo:docs/decisions/ADR-0024-codex-skill-delivery.md#sha256:9e4511247d0ed17075ec0b9b6435a612448c18e341fa4fd628c41d91688bc720"
  - "repo:tests/results/legacy/7d159c3524fa28e6fde6.md#sha256:6d07e8da05964857e1e492e6ea596cdc6567947c998ddc39dbcd0bd60ede8bab"
  - "repo:tests/results/legacy/f41c995a82c3db97f0a4.md#sha256:e19a2a3862f6936c502a46ebb443b99d3df992385d79b26e14856a52526e4193"
links: [global-onboarding, projection-upgrade-purge, public-skill-identity]
reviewed_revision: "git:34c4d6cef6c3992402b41c8a801f0b4e57fc68dd"
status: active
---

# Codex Skill Delivery and Automatic Cleanup

Codex 0.11.1 uses an active verified plugin for built-ins. User/project updates remove authenticated unchanged duplicates automatically. Changed or unproven files remain with reasons. Project-only Skills stay local; project updates restore local delivery if plugin verification fails. Deduplication requires matching invocation policy; missing or mismatched proof keeps local copies. Localized picker text is not policy.

Public test.6 current-user installation preserves knowledge, saved settings and foreign guidance. A fresh native admin server lists 28 unique Hive Skills with zero errors and no model calls. GUI and explicit/natural model invocation remain unverified.
