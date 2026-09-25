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
  - "repo:docs/guides/directive-context-recovery.md#sha256:20a1f22a3570c4dd8ad993c0ae04fee31ebcbc43646f90c72811501d30ea0438"
  - "repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e"
links: [project-policy-enforcement]
reviewed_revision: "git:1d36069531a894895a3517a2937666069cfe346e"
status: active
---

# Directive Recovery Across Compactions

Hive 0.11.0-test.8 implements opt-in approved Markdown recovery with a 4,096-byte cap, per-path edit guidance and protected-file checks. Restore events and child starts do not share a parent-session delivery cache or add model calls. Changed excerpts are not promoted; invalid configuration blocks guarded edits while removal remains available. Public binaries passed all 26 hook contract cases on Windows, macOS and Linux. Live Codex compaction, model adherence and usage comparison await activation authority. The goal is durable directives with minimal unnecessary enforcement and token overhead.
