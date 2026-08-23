---
schema_version: 1
pair_id: adversarial-judge
topic_slug: adversarial-judge
language: en
counterpart: ../ko/adversarial-judge.md
title: "Adversarial Judge Skill"
summary: "0.10.0 adds an explicit adversarial-judge step that prepares a clean-context host-native Judge request and reuses authenticated Hive quorum verification."
tags: [judge, skills, v0-10]
aliases: ["Adversarial review"]
sources:
  - "repo:crates/hive-cli/src/judge.rs#sha256:20dcfd35707b7571014ddc463601074179b42558e531c728d1c04bc634744ed0"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:ff4dfde9029c9024ab260f0366381e1a9bf1ce9d384a1db46b33d1cd842a5578"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/adversarial-judge-0.10.0.md#sha256:952b369d86a293d96c61f200379fac63590d70e66600bb6d43aea65bf4a130b8"
  - "repo:harness/skills/adversarial-judge/SKILL.md#sha256:9b8641f4c858698cb8959ed311cc2bcefb7764e1465b6109ec4343c2dc27f215"
  - "repo:schemas/adversarial-judge-host-receipt.schema.json#sha256:b6da86e2319a7df2b6921aa12eecf33c47beb6918a24cf322216f3e7d5d5946e"
links: [judge-verification, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:f91816a46d44d57929cb0b580ca32ff4caa95053"
status: active
---

# Adversarial Judge Skill

`adversarial-judge` prepares a clean-context request and provider-neutral dispatch envelope. The
active host launches the separate Judge. Read-only `hive judge receipt` binds the host launch and
result to the exact package, assignment, slot, Judge identity, model, effort, and verdict digest.
Findings remain diagnostic until the existing authenticated quorum authorizes acceptance. Hive
never calls a provider or spawns a process.
