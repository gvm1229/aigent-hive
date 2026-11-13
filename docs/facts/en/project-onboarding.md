---
schema_version: 1
pair_id: project-onboarding
topic_slug: project-onboarding
language: en
counterpart: ../ko/project-onboarding.md
title: "Project Onboarding"
summary: "Project setup inherits global choices and proposes local marker facts before confirmation."
tags: [onboarding, project]
aliases: ["Project setup"]
sources:
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:68d3ff4ada0f4f64af24a1f11b881c51ae650de438af02c7bf0b84a459c5cc0f"
  - "repo:docs/research/overhaul/OV151-007.md#sha256:69efaab07a728c908284c141ffa519f5fcf2b313a8cb0ec23d486e4bc489ff55"
links: [global-onboarding, shared-index]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Project Onboarding

Project setup inherits operational global choices and asks only unresolved essentials before the write preview. In 0.15.1 source, bounded local markers propose coding, docs, or general; custom choices remain available. Windows fixed-corpus detection matched 30/30, with no automatic acceptance. User confirmation precedes project writes. Public multi-OS acceptance remains separate. Origin: stage-four project detection implementation.
