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
links: [global-onboarding, projection-upgrade-purge, public-skill-identity]
reviewed_revision: "git:94fa2196febf288556780c7f68e6f752f15a6415"
status: active
---

# Codex Skill Delivery and Automatic Cleanup

The 0.11.1 implementation uses the verified active Codex plugin for built-in Skills. User and project updates remove authenticated duplicate copies automatically. Changed or unproven files stay intact with reasons. Project-only Skills stay local; project update restores local delivery when the plugin is unavailable. Registration and file tests do not prove actual Codex picker discovery or model invocation.

Deduplication also requires matching invocation policy. Policy mismatch or missing proof retains local delivery with a reason. Localized picker wording is not policy.
