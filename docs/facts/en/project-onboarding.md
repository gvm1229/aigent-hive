---
schema_version: 1
pair_id: project-onboarding
topic_slug: project-onboarding
language: en
counterpart: ../ko/project-onboarding.md
title: "Project Onboarding"
summary: "Project refresh updates facts while preserving a recorded kind choice."
tags: [onboarding, project]
aliases: ["Project setup"]
sources:
  - "repo:crates/hive-render/src/lib.rs#sha256:9859ba77fe6044c1da05931adedb5ffc1c9bb523b6953964438afb48a91c26fb"
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:68d3ff4ada0f4f64af24a1f11b881c51ae650de438af02c7bf0b84a459c5cc0f"
  - "repo:docs/research/overhaul/OV151-007.md#sha256:69efaab07a728c908284c141ffa519f5fcf2b313a8cb0ec23d486e4bc489ff55"
links: [global-onboarding, shared-index]
reviewed_revision: "git:43a894eb75b9fb5bd6327d47d97ce3665fa7a6e3"
status: active
---

# Project Onboarding

Project setup inherits operational global choices and asks unresolved essentials before the write preview. In 0.15.1 source, refresh preserves explicit, accepted-default, and current-version default project-kind decisions. Only missing decisions or pre-0.15.1 unconfirmed defaults may migrate general to coding. Detection updates project facts and reports a suggestion. Windows source regressions verify this boundary; public three-OS and real host Skill discovery acceptance remain unverified. Earlier 30-case implementation measurements remain in the linked stage evidence. Origin: stage-four review repair.
