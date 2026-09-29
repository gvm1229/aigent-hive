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
  - "repo:crates/hive-render/src/skill_delivery.rs#sha256:39a5ba630384bdd526f3fdec52dc51d09f9b8a9b00e7a66a5c7d2bf6fc1927ff"
  - "repo:docs/decisions/ADR-0024-codex-skill-delivery.md#sha256:a2c66b03a14bbef1564c50fcd2e541c31e1f956d3e29b9da5155a34c0ed42ffd"
links: [global-onboarding, projection-upgrade-purge, public-skill-identity]
reviewed_revision: "git:af8701b0ee9440a411a2f1701c643c28839ec4e8"
status: active
---

# Codex Skill Delivery and Automatic Cleanup

The 0.11.1 implementation uses the verified active Codex plugin for built-in Skills. User and project updates remove authenticated duplicate copies automatically. Changed or unproven files stay intact with reasons. Project-only Skills stay local; project update restores local delivery when the plugin is unavailable. Registration and file tests do not prove actual Codex picker discovery or model invocation.
