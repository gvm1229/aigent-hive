---
schema_version: 1
pair_id: directive-context-recovery
topic_slug: directive-context-recovery
language: en
counterpart: ../ko/directive-context-recovery.md
title: "Directive Recovery Across Compactions"
summary: "Approved excerpt recovery is implemented; live compaction adherence remains unverified."
tags: [context, hooks, policy]
aliases: []
sources:
  - "repo:docs/guides/directive-context-recovery.md#sha256:53c7c389d0fb7d20c61597bdb1388aa4c89c30f2871754eb51f835e9f424a8ee"
links: [project-policy-enforcement]
reviewed_revision: "git:7539f44b947b1fe0c90ea3d5c083022c99c9e341"
status: active
---

# Directive Recovery Across Compactions

Hive 0.11.0 implements opt-in, approved Markdown excerpt recovery with a 4,096-byte cap, per-path edit guidance, and explicit protected-file checks. Each compaction restores again without parent-session deduplication or extra model calls. Changed excerpts are not promoted; invalid approved configuration blocks guarded edits while removal remains available. Windows CLI replay verifies delivery and checks, not live-host compaction or model adherence. The goal is durable directives with low token overhead.
