---
schema_version: 1
pair_id: adversarial-judge
topic_slug: adversarial-judge
language: ko
counterpart: ../en/adversarial-judge.md
title: "Adversarial Judge Skill"
summary: "Clean-context host-native Judge 요청을 준비하고 기존 authenticated quorum 검증을 재사용하는 0.10.0 명시 단계"
tags: [judge, skills, v0-10]
aliases: ["Adversarial review"]
sources:
  - "repo:crates/hive-cli/src/judge.rs#sha256:20dcfd35707b7571014ddc463601074179b42558e531c728d1c04bc634744ed0"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:39231490f4083cba9cfaba64dbf265045ccd9cbcada90cd3646cdbd936932c19"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/adversarial-judge-0.10.0.md#sha256:952b369d86a293d96c61f200379fac63590d70e66600bb6d43aea65bf4a130b8"
  - "repo:harness/skills/adversarial-judge/SKILL.md#sha256:9b8641f4c858698cb8959ed311cc2bcefb7764e1465b6109ec4343c2dc27f215"
  - "repo:schemas/adversarial-judge-host-receipt.schema.json#sha256:b6da86e2319a7df2b6921aa12eecf33c47beb6918a24cf322216f3e7d5d5946e"
links: [judge-verification, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:f91816a46d44d57929cb0b580ca32ff4caa95053"
status: active
---

# Adversarial Judge Skill

- 새 역할: Clean-context request·provider-neutral dispatch envelope 준비
- 실제 launch: Active host 소유
- `hive judge receipt`: Package·assignment·slot·Judge identity·model·effort·verdict digest read-only binding
- 판정: Finding은 diagnostic, 기존 authenticated quorum 통과 뒤에만 acceptance authority
- 금지: Provider 호출·credential·Hive direct process spawn
