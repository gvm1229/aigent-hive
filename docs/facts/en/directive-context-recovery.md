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
  - "repo:docs/research/directive-context-instrumented-tests-2026-09-26.md#sha256:e2a20c1a87519399eca28095c6a4de7e267a3fe87ca9f4c103ab252a5b07c9ab"
links: [project-policy-enforcement]
reviewed_revision: "git:b90d51eb7d60c5f5f6b555d329cfef7800330c60"
status: active
---

# Directive Recovery Across Compactions

Test.8 restores approved Markdown within 4,096 bytes, with path guidance and file guards. Every compaction/child start restores without parent deduplication or extra model calls. Changed text is not promoted; invalid settings block guarded edits, with removal available. Public binaries passed 26 contracts per OS. Windows manual compaction reached ten with file checks. Trusted Hive hooks restored after three automatic compactions in one turn. A removal/reapply fix passed 27 checks, one privilege skip. Paired text input saved 1,662 tokens; billing and broad adherence remain unproven. Goal: durable rules with low overhead.
