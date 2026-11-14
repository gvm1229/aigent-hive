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
  - "repo:crates/hive-render/src/skill_delivery.rs#sha256:760338514e3b24cf0b66da4abe9af90b00aa1de2c9744741d878638164701c3a"
  - "repo:docs/decisions/ADR-0024-codex-skill-delivery.md#sha256:670cfb990aa37bdc659afe890f601b5740954a7c69fb16d4223cb8e541279097"
  - "repo:docs/research/wproject-custom-ship-0.11.1.md#sha256:bfc275b9392b1ba2fcac66b3217f40a77a9d2fff264bf67ba3d172ad2da23004"
  - "repo:tests/results/legacy/7d159c3524fa28e6fde6.md#sha256:6d07e8da05964857e1e492e6ea596cdc6567947c998ddc39dbcd0bd60ede8bab"
  - "repo:tests/results/legacy/f41c995a82c3db97f0a4.md#sha256:e19a2a3862f6936c502a46ebb443b99d3df992385d79b26e14856a52526e4193"
links: [global-onboarding, projection-upgrade-purge, public-skill-identity]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Codex Skill Delivery and Automatic Cleanup

Codex 0.11.1 uses an active verified plugin for built-ins. User updates remove authenticated unchanged duplicates; project updates only in compact layout, so full-layout copies stay on every machine. Changed or unproven files remain with reasons. Project-only Skills stay local; project updates restore local delivery if plugin verification fails. Deduplication requires matching invocation policy; missing or mismatched proof keeps local copies.

Public test.6 current-user installation preserves knowledge, saved settings and foreign guidance. A fresh native admin server lists 28 unique Hive Skills with zero errors and no model calls. Fresh-chat explicit and natural user-setup calls pass validation. Project Skill use is a separate check.
