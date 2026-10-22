---
schema_version: 1
pair_id: hive-desktop-feasibility
topic_slug: hive-desktop-feasibility
language: en
counterpart: ../ko/hive-desktop-feasibility.md
title: "Desktop Harness Feasibility Research"
summary: "Desktop research separates an external-engine app from a Hive-owned agent engine."
tags: [architecture, desktop, research, runtime]
aliases: [Hive Desktop]
sources:
  - "repo:docs/research/hive-desktop/README.md#sha256:a17b9c9a2a5ebace89510af320604ee0c1644f90e58793f712c1484e7d89b100"
  - "repo:docs/research/hive-desktop/changes.md#sha256:9eac567a47a10efb98e7897a8004223b0ec37a43d888354bf557fb9595d779e5"
  - "repo:docs/research/hive-desktop/runtime.md#sha256:46b736a53674c87f21e847f13eb41c0964619c0fb4964ead224505113776b323"
links: [orchestration-ownership, product-non-goals]
reviewed_revision: "git:88f47770af645627980e3627b02d334dd2ce9001"
status: active
---

# Desktop Harness Feasibility Research

The user requested a complete desktop-app feasibility study. The [report](../../research/hive-desktop/README.md) separates an external-engine app from a Hive-owned agent engine. Tauri 2, React, TypeScript, and shared Rust services are research recommendations.

Windows code inspection and official documents support the research. No desktop build, model execution, authentication access, performance, schedule, or macOS/Linux qualification is proven. Runtime-policy changes and a new product version remain unapproved.
