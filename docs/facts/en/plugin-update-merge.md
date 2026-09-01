---
schema_version: 1
pair_id: plugin-update-merge
topic_slug: plugin-update-merge
language: en
counterpart: ../ko/plugin-update-merge.md
title: "Plugin Update Merge"
summary: "Signed historical base bytes drive local-priority three-way projection updates."
tags: [merge, plugin, update]
aliases: ["Projection upgrade merge"]
sources:
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:a78f7c3acbe764bc04916912e9fbb15bd9c5b90275db7f376add543439f1e90a"
links: [project-onboarding, update-transaction]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Plugin Update Merge

An authenticated historical base distinguishes unmodified and user-modified
projections. Unmodified files accept exact replacement; modified files keep
overlapping local edits and receive only non-conflicting incoming hunks.
