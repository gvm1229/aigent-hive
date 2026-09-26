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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/decisions/product-release-decisions.md#sha256:9f234ef3fede8030ab6ad57fa4b560a71f30f468622c4e4ad56b83864d0e05ce"
  - "repo:docs/plans/active/adversarial-judge-0.10.0.md#sha256:952b369d86a293d96c61f200379fac63590d70e66600bb6d43aea65bf4a130b8"
  - "repo:harness/skills/adversarial-judge/SKILL.md#sha256:9b8641f4c858698cb8959ed311cc2bcefb7764e1465b6109ec4343c2dc27f215"
  - "repo:schemas/adversarial-judge-host-receipt.schema.json#sha256:b6da86e2319a7df2b6921aa12eecf33c47beb6918a24cf322216f3e7d5d5946e"
links: [judge-verification, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Adversarial Judge Skill

`adversarial-judge` prepares a clean-context request and provider-neutral dispatch envelope. The
active host launches the separate Judge. Read-only `hive judge receipt` binds the host launch and
result to the exact package, assignment, slot, Judge identity, model, effort, and verdict digest.
Findings remain diagnostic until the existing authenticated quorum authorizes acceptance. Hive
never calls a provider or spawns a process.
