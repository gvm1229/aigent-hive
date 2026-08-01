---
schema_version: 1
pair_id: shared-index-target-safety
topic_slug: shared-index-target-safety
language: en
counterpart: ../ko/shared-index-target-safety.md
title: "Shared Index Target Path Safety"
summary: "Shared knowledge commands reject linked consumer targets before canonicalization and ignore the retired project-local stale marker."
tags: [index, security, symlink]
aliases: ["legacy stale marker", "shared knowledge target guard"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:219a260a79a0e605457294a1d45c4f803b52510e0d148be2ebd1ee600a959a5f"
links: [knowledge-storage, shared-index]
reviewed_revision: "git:f57bd3f7c3057370a2afc5094f6b1bb1847bdf02"
status: active
---

# Shared Index Target Path Safety

Shared knowledge queries and mutations inspect the supplied consumer target and
reject any existing symbolic-link component before canonicalizing the path. The
retired project-local `.hive/index/.stale` marker is inert: shared queries and
rebuilds neither follow nor mutate it. Acceptance: linked registered-project
aliases fail without project changes, while a linked legacy marker remains
unchanged during successful shared-index operations. Origin: cross-platform CI
exposed Windows link tests that a non-elevated local Windows host could not run.
