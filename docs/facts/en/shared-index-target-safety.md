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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:f8920322c1f918b16e9b2df7c1b3a29867cbd4c6cc95b82caa33016d63faab47"
links: [knowledge-storage, shared-index]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
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
