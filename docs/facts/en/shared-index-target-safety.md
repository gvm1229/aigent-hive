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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:fb2a64a7cafaf921a5252a65b042ade24ab03b7dcee5eec52dc7b895b5b92df5"
links: [knowledge-storage, shared-index]
reviewed_revision: "git:52e63238cb6063312240d1c9b5ed1006d8a23a7b"
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
